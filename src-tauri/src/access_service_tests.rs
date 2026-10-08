//! The accounts with the real database: the first account, entering and its mistakes, the administration rules,
//! the recovery code and the deletions that wait for the administrator.

use super::*;
use crate::domain::access::{Need, Permission};
use crate::domain::profile::{InstitutionInput, InstitutionKind, ProfileInput};
use crate::hr::domain::person::PersonData;
use crate::storage::open_encrypted;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const ADMIN_PASSWORD: &str = "una clave larga";

fn conn() -> (tempfile::TempDir, Connection) {
    let dir = tempfile::tempdir().unwrap();
    let c = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    (dir, c)
}

fn with_admin(c: &mut Connection) -> (Session, SetupOutcome) {
    let session = Session::default();
    let out = setup_admin(c, &session, &Attempts::default(), "Jaime Caballero", "jaime", ADMIN_PASSWORD, None).unwrap();
    (session, out)
}

fn admin(session: &Session) -> CurrentUser {
    session.current().unwrap()
}

fn events(c: &Connection) -> Vec<(String, Option<String>, String)> {
    c.prepare("SELECT event, actor_id, details_json FROM audit_log ORDER BY rowid").unwrap().query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap().collect::<Result<_, _>>().unwrap()
}

/// A person of the staff and an account for them as direction, already with their own password.
fn manager(c: &mut Connection, admin_session: &Session) -> (String, Session) {
    with_profile(c);
    let pos = crate::staff_service::overview(c).unwrap().hr.positions[0].position.id.clone();
    let data = PersonData { first_names: "Rosa María".into(), last_name_1: Some("Hernández".into()), position_id: Some(pos), modality: "indefinite".into(), status: "active".into(), ..Default::default() };
    let crate::staff_service::PersonOutcome::Saved { person, .. } = crate::staff_service::save_person(c, None, data).unwrap() else { panic!("saved") };
    let a = admin(admin_session);
    create_user(c, &a, &NewAccount { person_id: Some(&person.id), display_name: "", username: "rosa.hernandez", role: Role::Manager, temporary_password: "temporal123" }).unwrap();
    let session = Session::default();
    let LoginOutcome::Ok { session: view } = login(c, &session, "rosa.hernandez", "temporal123").unwrap() else { panic!("in") };
    assert!(view.must_change_password, "a temporary password is changed first");
    change_password(c, &session, "temporal123", "mi clave propia").unwrap();
    (person.id, session)
}

fn with_profile(c: &mut Connection) {
    if crate::storage::profile::load_current(c).unwrap().is_none() {
        let input = ProfileInput { institution: InstitutionInput { name: "Asilo Ficticio".into(), kind: InstitutionKind::ElderlyHome, ..Default::default() }, ..Default::default() };
        crate::storage::profile::save(c, &input).unwrap();
    }
}

#[test]
fn the_first_account_is_the_administrator_and_it_can_be_made_only_once() {
    let (_d, mut c) = conn();
    let session = Session::default();
    assert!(status(&c, &session).unwrap().needs_setup);
    let (session, out) = with_admin(&mut c);
    assert_eq!(out.session.role, Role::Admin);
    assert_eq!(out.recovery_code.len(), 19, "XXXX-XXXX-XXXX-XXXX: {}", out.recovery_code);
    let st = status(&c, &session).unwrap();
    assert!(!st.needs_setup && st.session.is_some());
    assert!(matches!(setup_admin(&mut c, &Session::default(), &Attempts::default(), "Otra", "otra", ADMIN_PASSWORD, None), Err(ServiceError::Access("already_set_up"))));
    let stored: String = c.query_row("SELECT password_hash FROM app_user", [], |r| r.get(0)).unwrap();
    assert!(stored.starts_with("$argon2id$") && !stored.contains(ADMIN_PASSWORD));
    let (event, actor, _) = events(&c).into_iter().next().unwrap();
    assert_eq!((event.as_str(), actor.is_some()), ("access.setup", true));
}

#[test]
fn a_computer_with_a_pin_asks_it_before_making_the_administrator_and_then_the_pin_retires() {
    let (_d, mut c) = conn();
    let attempts = Attempts::default();
    crate::security_service::pin_set(&c, &attempts, "4821", None).unwrap();
    assert!(status(&c, &Session::default()).unwrap().setup_needs_pin);
    assert!(matches!(setup_admin(&mut c, &Session::default(), &attempts, "Jaime", "jaime", ADMIN_PASSWORD, Some("1111")), Err(ServiceError::Access("wrong_pin"))));
    setup_admin(&mut c, &Session::default(), &attempts, "Jaime", "jaime", ADMIN_PASSWORD, Some("4821")).unwrap();
    assert!(!crate::security_service::pin_enabled(&c).unwrap());
}

#[test]
fn wrong_passwords_make_the_account_wait_and_say_nothing_of_which_part_was_wrong() {
    let (_d, mut c) = conn();
    let (s, _) = with_admin(&mut c);
    logout(&c, &s).unwrap();
    let session = Session::default();
    assert!(matches!(login(&c, &session, "nadie", "x").unwrap(), LoginOutcome::Wrong));
    for _ in 0..4 {
        assert!(matches!(login(&c, &session, "jaime", "mala clave").unwrap(), LoginOutcome::Wrong));
    }
    assert!(matches!(login(&c, &session, "jaime", "mala clave").unwrap(), LoginOutcome::Waiting { wait_secs: 30 }));
    assert!(matches!(login(&c, &session, "JAIME", ADMIN_PASSWORD).unwrap(), LoginOutcome::Waiting { .. }), "even the right one waits");
    assert!(session.current().is_none());
    c.execute("UPDATE app_user SET locked_until = 0", []).unwrap();
    assert!(matches!(login(&c, &session, "Jaime", ADMIN_PASSWORD).unwrap(), LoginOutcome::Ok { .. }));
    let all = events(&c);
    assert!(all.iter().any(|(e, _, _)| e == "auth.locked"));
    assert!(all.iter().all(|(_, _, d)| !d.contains("mala") && !d.contains(ADMIN_PASSWORD)), "no password in the log");
}

#[test]
fn a_temporary_password_is_changed_before_anything_else() {
    let (_d, mut c) = conn();
    let (s, _) = with_admin(&mut c);
    let a = admin(&s);
    create_user(&c, &a, &NewAccount { person_id: None, display_name: "Contador Ficticio", username: "contador", role: Role::Manager, temporary_password: "temporal123" }).unwrap();
    let session = Session::default();
    login(&c, &session, "contador", "temporal123").unwrap();
    let user = session.current().unwrap();
    assert_eq!(check(Some(user.clone()), false, Need::Permission(Permission::Use)), Err("must_change_password"));
    assert!(check(Some(user), false, Need::Session).is_ok(), "changing the password is allowed");
    assert!(matches!(change_password(&c, &session, "otra", "nueva clave 1"), Err(ServiceError::Access("wrong_password"))));
    assert!(matches!(change_password(&c, &session, "temporal123", "temporal123"), Err(ServiceError::Access("password_same"))));
    let view = change_password(&c, &session, "temporal123", "nueva clave 1").unwrap();
    assert!(!view.must_change_password);
    assert!(check(session.current(), false, Need::Permission(Permission::Use)).is_ok());
}

#[test]
fn the_check_of_every_command() {
    let manager = CurrentUser { id: "u".into(), username: "rosa".into(), display_name: "Rosa".into(), role: Role::Manager, must_change_password: false };
    let admin = CurrentUser { role: Role::Admin, ..manager.clone() };
    assert_eq!(check(None, false, Need::Permission(Permission::Use)), Err("not_signed_in"));
    assert!(check(None, false, Need::Open).is_ok());
    assert_eq!(check(Some(manager.clone()), true, Need::Permission(Permission::Use)), Err("session_locked"));
    assert!(check(Some(manager.clone()), false, Need::Permission(Permission::Use)).is_ok());
    assert!(check(Some(manager.clone()), false, Need::DeleteOrRequest).is_ok(), "it passes and becomes a request");
    for p in [Permission::Settings, Permission::Administer, Permission::Delete] {
        assert_eq!(check(Some(manager.clone()), false, Need::Permission(p)), Err("access_denied"), "{p:?}");
        assert!(check(Some(admin.clone()), false, Need::Permission(p)).is_ok(), "{p:?}");
    }
}

impl PartialEq for CurrentUser {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[test]
fn administration_rules_keep_one_administrator_and_one_account_per_person() {
    let (_d, mut c) = conn();
    let (s, _) = with_admin(&mut c);
    let a = admin(&s);
    let (person, _) = manager(&mut c, &s);
    assert!(matches!(
        create_user(&c, &a, &NewAccount { person_id: None, display_name: "Otra", username: "otra.admin", role: Role::Admin, temporary_password: "temporal123" }),
        Err(ServiceError::Access("role_not_allowed"))
    ));
    assert!(matches!(
        create_user(&c, &a, &NewAccount { person_id: Some(&person), display_name: "", username: "rosa2", role: Role::Manager, temporary_password: "temporal123" }),
        Err(ServiceError::Access("person_has_account"))
    ));
    assert!(matches!(
        create_user(&c, &a, &NewAccount { person_id: None, display_name: "X", username: "Rosa.Hernandez", role: Role::Manager, temporary_password: "temporal123" }),
        Err(ServiceError::Access("username_taken"))
    ));
    assert!(matches!(update_user(&c, &a, &a.id, Role::Manager, true), Err(ServiceError::Access("last_admin"))));
    assert!(matches!(update_user(&c, &a, &a.id, Role::Admin, false), Err(ServiceError::Access("last_admin"))));

    let overview = admin_overview(&c).unwrap();
    let rosa = overview.users.iter().find(|u| u.username == "rosa.hernandez").unwrap();
    assert_eq!(rosa.display_name, "Rosa María Hernández", "the name comes from the staff record");
    assert!(overview.people.iter().any(|p| p.person_id == person && p.user_id.as_deref() == Some(rosa.id.as_str())));
    let off = update_user(&c, &a, &rosa.id, Role::Manager, false).unwrap();
    assert!(!off.users.iter().find(|u| u.id == rosa.id).unwrap().active);
    assert!(matches!(login(&c, &Session::default(), "rosa.hernandez", "mi clave propia").unwrap(), LoginOutcome::Disabled));
}

#[test]
fn a_person_who_leaves_the_institution_loses_access() {
    let (_d, mut c) = conn();
    let (s, _) = with_admin(&mut c);
    let (person, _) = manager(&mut c, &s);
    let mut data = crate::hr::service::get_person(&c, &person).unwrap().unwrap().data;
    data.status = "left".into();
    data.left_date = Some("2026-10-01".into());
    crate::staff_service::save_person(&mut c, Some(&person), data).unwrap();
    assert!(matches!(login(&c, &Session::default(), "rosa.hernandez", "mi clave propia").unwrap(), LoginOutcome::Disabled));
}

#[test]
fn the_recovery_code_opens_once_and_is_renewed() {
    let (_d, mut c) = conn();
    let (s, out) = with_admin(&mut c);
    logout(&c, &s).unwrap();
    let session = Session::default();
    assert!(matches!(recover(&c, &session, "jaime", "AAAA-BBBB-CCCC-DDDD", "nueva clave larga"), Err(ServiceError::Access("recovery_wrong"))));
    let lower = out.recovery_code.to_lowercase().replace('-', " ");
    let again = recover(&c, &session, "jaime", &lower, "nueva clave larga").unwrap();
    assert_ne!(again.recovery_code, out.recovery_code);
    assert!(matches!(recover(&c, &Session::default(), "jaime", &out.recovery_code, "otra clave larga"), Err(ServiceError::Access("recovery_wrong"))), "the old code no longer works");
    assert!(matches!(login(&c, &Session::default(), "jaime", "nueva clave larga").unwrap(), LoginOutcome::Ok { .. }));
}

#[test]
fn the_locked_screen_opens_only_with_the_password_of_who_was_inside() {
    let (_d, mut c) = conn();
    let (s, _) = with_admin(&mut c);
    lock(&s);
    assert!(status(&c, &s).unwrap().session.unwrap().locked);
    assert_eq!(check(s.current(), true, Need::Permission(Permission::Use)), Err("session_locked"));
    assert!(matches!(unlock(&c, &s, "otra").unwrap(), LoginOutcome::Wrong));
    assert!(matches!(unlock(&c, &s, ADMIN_PASSWORD).unwrap(), LoginOutcome::Ok { .. }));
    assert!(!s.state.lock().unwrap().locked);
}

#[test]
fn a_deletion_asked_by_direction_hides_the_person_until_the_administrator_decides() {
    let (_d, mut c) = conn();
    let (s, _) = with_admin(&mut c);
    let a = admin(&s);
    let (_, rosa_session) = manager(&mut c, &s);
    let rosa = rosa_session.current().unwrap();
    let pos = crate::staff_service::overview(&c).unwrap().hr.positions[0].position.id.clone();
    let data = PersonData { first_names: "Ana".into(), position_id: Some(pos), modality: "indefinite".into(), status: "active".into(), pay_amount_mxn: Some(9_000), ..Default::default() };
    let crate::staff_service::PersonOutcome::Saved { person, .. } = crate::staff_service::save_person(&mut c, None, data).unwrap() else { panic!() };
    let payroll = |c: &Connection| crate::storage::profile::load_current(c).unwrap().unwrap().input.staff.iter().map(|l| l.monthly_salary_mxn.unwrap_or(0) * l.count).sum::<i64>();
    assert_eq!(payroll(&c), 9_000);

    request_deletion(&mut c, &rosa, DeletionKind::HrPerson, &person.id).unwrap();
    assert!(!crate::staff_service::overview(&c).unwrap().hr.people.iter().any(|p| p.id == person.id), "gone from the list for everyone");
    assert_eq!(payroll(&c), 0, "and from the sums");
    assert!(matches!(request_deletion(&mut c, &rosa, DeletionKind::HrPerson, &person.id), Err(ServiceError::Access("already_requested"))));
    let pending = admin_overview(&c).unwrap().pending;
    assert_eq!((pending.len(), pending[0].target_label.as_str(), pending[0].requested_by_name.as_str()), (1, "Ana", "Rosa María Hernández"));

    // rejected: it comes back
    resolve_request(&mut c, &a, &pending[0].id, false).unwrap();
    assert!(crate::staff_service::overview(&c).unwrap().hr.people.iter().any(|p| p.id == person.id));
    assert_eq!(payroll(&c), 9_000);

    // asked again and approved: deleted for good
    request_deletion(&mut c, &rosa, DeletionKind::HrPerson, &person.id).unwrap();
    let id = admin_overview(&c).unwrap().pending[0].id.clone();
    let after = resolve_request(&mut c, &a, &id, true).unwrap();
    assert!(after.pending.is_empty() && after.resolved.iter().any(|r| r.status == "approved"));
    assert!(crate::hr::service::get_person(&c, &person.id).unwrap().is_none());
    let kinds: Vec<String> = events(&c).into_iter().map(|e| e.0).collect();
    for k in ["request.created", "request.rejected", "request.approved", "hr.person_deleted"] {
        assert!(kinds.iter().any(|e| e == k), "{k}");
    }
}

#[test]
fn a_hidden_field_keeps_its_values_and_a_document_hides_too() {
    use crate::domain::roster::{Data, Entity};
    use crate::storage::roster::FieldInput;
    let (_d, mut c) = conn();
    let (s, _) = with_admin(&mut c);
    let (_, rosa_session) = manager(&mut c, &s);
    let rosa = rosa_session.current().unwrap();
    let fields = crate::roster_service::save_field(&c, Entity::Beneficiary, &FieldInput { key: None, title: "Alergias".into(), kind: crate::domain::roster::FieldKind::Text, options: vec![] }).unwrap();
    let key = fields.iter().find(|f| f.title == "Alergias").unwrap().key.clone();
    let mut data: Data = [("full_name", "Luz"), ("category", "Mujeres adultas mayores")].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    data.insert(key.clone(), "nueces".into());
    let change = crate::roster_service::save_entry(&mut c, Entity::Beneficiary, None, &data).unwrap();
    let id = change.entries[0].id.clone();

    request_deletion(&mut c, &rosa, DeletionKind::RosterField, &key).unwrap();
    // the record is edited while the field is hidden: its value survives
    data.remove(&key);
    crate::roster_service::save_entry(&mut c, Entity::Beneficiary, Some(&id), &data).unwrap();
    assert_eq!(crate::storage::roster::stored(&c, &id).unwrap().unwrap().get(&key).map(String::as_str), Some("nueces"));

    let doc = crate::storage::documents::add_text_document(&mut c, "internal", "Reglamento", "Texto del reglamento interno.", 0).unwrap();
    request_deletion(&mut c, &rosa, DeletionKind::Document, &doc.id).unwrap();
    assert!(crate::storage::documents::list_institution(&c).unwrap().is_empty());
}
