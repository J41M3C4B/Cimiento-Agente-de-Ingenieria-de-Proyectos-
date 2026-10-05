"""Genera positivos.txt y negativos.txt con identificadores INVENTADOS de formato válido.
Uso: python generar.py   (formato de línea: tipo | texto; '#' = comentario)"""
import random

random.seed(2026)
DICT = "0123456789ABCDEFGHIJKLMNÑOPQRSTUVWXYZ"
CONS = "BCDFGHJKLMNPQRSTVWXYZ"


def curp(letters4, date6, sex, state, cons3, homo):
    base = letters4 + date6 + sex + state + cons3 + homo
    s = sum(DICT.index(c) * (18 - i) for i, c in enumerate(base))
    return base + str((10 - s % 10) % 10)


def clabe(prefix17):
    w = [3, 7, 1]
    s = sum((int(d) * w[i % 3]) % 10 for i, d in enumerate(prefix17))
    return prefix17 + str((10 - s % 10) % 10)


def luhn_complete(prefix):
    for d in range(10):
        n = prefix + str(d)
        t = 0
        for i, ch in enumerate(reversed(n)):
            x = int(ch)
            if i % 2 == 1:
                x = x * 2 - 9 if x * 2 > 9 else x * 2
            t += x
        if t % 10 == 0:
            return n


c1 = curp("LOPM", "800101", "M", "DF", "RZN", "0")
c2 = curp("GAHJ", "150930", "H", "JC", "RRS", "A")
c3 = curp("RAMC", "921215", "M", "NL", "NRL", "0")
cl1 = clabe("01218000123456789")
cl2 = clabe("01418000987654321")
card1 = luhn_complete("411111111111111")
card2 = luhn_complete("550000000000000")
nss1 = luhn_complete("4285901234")
nss2 = luhn_complete("1198765432")

pos = f"""# Casos que el escaner DEBE detectar. Formato: tipo | texto. Todo inventado.
curp | Su CURP es {c1} y vive en la casa.
curp | CURP: {c2.lower()} (minusculas)
curp | La nina tiene la curp {c3}.
curp | Datos: {c1[:4]} {c1[4:]} separado por espacio no cuenta, pero esta si: {c3}
rfc_person | RFC de la senora: LOPM800101AB3
rfc_person | rfc personal gahj150930x12 del familiar
rfc_company | La proveedora FCH950312K45 emitio la factura.
voter_key | Clave de elector LPMRMN80010109H400 de la residente.
voter_key | INE clave GRHRJS15093014M123
phone | Llamar al 55 1234 5678 para avisar.
phone | Telefono de la hija: (33) 3456-7890
phone | Contacto +52 81 8765 4321
phone | Su celular es 5512345678.
email | Escribale a maria.lopez@example.com por favor.
email | correo: JUAN_PEREZ@correo.mx
clabe | Deposito a la CLABE {cl1}
clabe | cuenta clabe {cl2} del familiar
card | Tarjeta {card1[:4]} {card1[4:8]} {card1[8:12]} {card1[12:]}
card | pago con {card2}
nss | Numero de seguro social {nss1}
nss | NSS {nss2} de la senora
person_name | La senora Rosa Hernandez Lopez vive en la habitacion 4.
person_name | Atendemos a Juan Perez Garcia desde enero.
person_name | La niña Camila llego en marzo.
person_name | Doña Josefa Ramírez cumple años hoy.
person_name | el residente Don Aurelio Martínez
health_context | El señor Juan Pérez García padece diabetes.
health_context | La señora de la cama 4 padece alzheimer.
health_context | Una de las niñas tiene epilepsia y toma medicamento.
health_context | La residente tiene diagnóstico de hipertensión.
health_context | María López García toma medicamento para la presión.
"""
open("positivos.txt", "w", encoding="utf-8").write(pos)

neg = """# Frases que el escaner NO debe marcar. Todo inventado.
Atendemos a 18 adultos mayores con dependencia alta.
12 de los 18 residentes viven con diabetes.
Seis de las niñas usan lentes y 3 requieren tratamiento dental.
El proyecto busca reducir el riesgo de caídas en 18 personas.
Casa Hogar San José
Asilo Santa María de Guadalupe
Fundación Esperanza y Vida, A.C.
Nacional Monte de Piedad
Junta de Asistencia Privada del Distrito Federal
Folio de convocatoria JAP-2026-000123
Monto solicitado: $184,500.00 pesos.
El presupuesto total es de 150000 pesos.
Fecha límite de entrega: 15/11/2026.
Código postal 06600, colonia Centro.
Registro de la solicitud 2026-10-01.
La institución cuenta con 3 enfermeras y 2 cocineras de planta.
Los baños no tienen barras de apoyo ni regaderas con asiento.
Se requieren 4 regaderas y 6 barras de apoyo.
Los residentes con alzheimer requieren supervisión continua.
La Madre Superiora autorizó el proyecto.
Capacidad total: 25 camas, 18 ocupadas.
Horario de atención de 9:00 a 17:00 horas.
Objetivo 1: mejorar la seguridad de las personas.
Cantidad 100,000 y 250,000 pesos en dos ministraciones.
Duración del proyecto: 12 meses.
"""
open("negativos.txt", "w", encoding="utf-8").write(neg)
print(c1, c2, c3, cl1, cl2, card1, card2, nss1, nss2)
