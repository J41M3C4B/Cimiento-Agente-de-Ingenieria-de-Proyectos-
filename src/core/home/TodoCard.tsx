import { Icon } from "../../components/icons";
import { es } from "../../i18n/es-MX";

const t = es.home.todo;

/** «Por hacer»: the tray for the person's own pending things. It is only the frame for now; the tasks come later. */
export function TodoCard() {
  return (
    <section className="todo-card" aria-labelledby="home-todo">
      <h2 id="home-todo" className="fig-head">
        <Icon name="check" size={20} />
        {t.title}
      </h2>
      <p className="text-ui text-ink-2">{t.empty}</p>
    </section>
  );
}
