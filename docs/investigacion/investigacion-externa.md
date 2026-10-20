# Investigación externa: fricción de las OSC, UX con IA, arquitectura de IA en un ERP y privacidad

Fecha: 2026-10-20. Investigación en la web para la arquitectura de IA de SociAI. Las opiniones **[O]** se escribieron sobre el código de ese día; no son decisiones.

**Cómo leer este reporte.**
- **[H]** es un hecho con fuente (URL al lado).
- **[O]** es mi opinión o inferencia para SociAI.
- **[V]** marca una cifra que viene de un vendedor o de una fuente secundaria que no pude verificar en el original.

**Limitación principal.** Casi no hay estudios públicos y recientes sobre la digitalización de las OSC **mexicanas**. No encontré ninguno de Alternativas y Capacidades, de Cemefi ni de Fundación Telefónica con cifras de adopción tecnológica. Por eso uso tres cosas:
- datos duros del sector mexicano (registro, SAT, INEGI, la guía de cumplimiento de Cemefi de 2026);
- encuestas globales con buena muestra (Salesforce, TechSoup y Connect Humanity);
- un estudio de LACNIC para Latinoamérica.

---

## 1. Fricción de las OSC con la tecnología

### 1.1 El tamaño y la forma del sector en México

- **[H]** En enero de 2025 había 49,187 OSC registradas: 45,269 con CLUNI y 10,469 donatarias autorizadas (6,551 con ambos registros). Fuente: Cemefi, «Información del Tercer Sector», con el Registro Federal de OSC (28-01-2025) y el Anexo 14 de la RMF 2025. https://www.cemefi.org/wp-content/uploads/2025/02/Informacion-Tercer-Sector-Cemefi.pdf
  - La tabla por actividad sugiere que la categoría más grande de donatarias es la asistencial (unas 6,600 en 2025), pero la extracción del PDF desalinea las etiquetas. Hay que confirmarlo en el original.
- **[H]** El sector no lucrativo generó 3,073,041 puestos de trabajo en 2023, el 7.8 % de la economía. Su PIB, 3.1 % del nacional, incluye la valoración del trabajo voluntario. https://inegi.org.mx/contenidos/saladeprensa/boletines/2024/CSISFLM/CSISFLM2023.pdf
- **[H][V]** En 2022 solo el 58.7 % de esos puestos eran remunerados; el resto era voluntario. La cifra es de Forbes citando a INEGI. https://forbes.com.mx/?p=1174045
- **[O]** Más de 40 % del trabajo es voluntario. Eso significa rotación alta y conocimiento que se va con la persona. El programa debe guardar «la memoria de la institución», no depender de quien la capturó.

### 1.2 La carga administrativa: el dolor más concreto y medible

- **[H]** La «Guía de cumplimiento legal, fiscal y contable para OSC en México» (Espacio de Donantes, Red Ensamble y Cemefi, enero 2026) dice que el marco normativo «se ha vuelto considerablemente más complejo». Para las organizaciones «de menor capacidad técnica o presupuestal» representa «un reto desproporcionado». https://www.cemefi.org/wp-content/uploads/2026/03/GuiaDONATARIAS.pdf
- **[H]** Lo que pide esa guía para mantener la autorización de donataria:
  - **Constancia de acreditación de actividades.** Su vigencia es de tres años, salvo que diga menos, y «comúnmente de un año». Si vence, el SAT puede iniciar la revocación.
  - **Declaración anual.** Antes del 15 de febrero.
  - **Declaración de transparencia.** En mayo.
  - **Las dos declaraciones deben coincidir plenamente.**
  - **Obligaciones de la LFPIORPI (ley antilavado).** Se reformaron en julio de 2025.

  Fuente: la misma guía.
- **[H]** Las OSC con CLUNI presentan su informe anual a más tardar el 31 de enero, aunque no hayan recibido recursos públicos (art. 7 fr. IV de la LFFAROSC). https://www.cemefi.org/images/pdf/2021_obligaciones_donatarias.pdf y https://www.cemefi.org/images/pdf/cemefi-informa-guia-informe%20anual-2020-cluni.pdf
- **[H]** Una donataria no puede destinar más del 5 % de los donativos a gastos de administración. Fuente: la misma tabla de Cemefi de 2021.
- **[H][V]** Revocaciones del SAT: 270 donatarias en 2025, de un padrón de más de 10 mil; 454 en 2023 y 349 en 2024. Entre las causas está no presentar documentación que respalde las actividades.
  - https://www.unotv.com/negocios/sat-defiende-revocacion-de-donatarias-fueron-270-de-10-mil/
  - https://idconline.mx/fiscal-contable/2026/03/25/sat-aclara-motivos-de-revocacion-a-donatarias-por-incumplimientos-fiscales
  - Las cifras de 2023 y 2024 son de Forbes citando al SAT: https://forbes.com.mx/?p=1386532
- **[H][V]** 923 organizaciones no presentaron la declaración de transparencia del ejercicio 2022. La multa reportada es de $102,060. https://www.taxtodaymexico.com/?p=7861 y https://www.elcontribuyente.mx/2024/03/cuales-son-las-obligaciones-de-las-donatarias-autorizadas/
- **[O]** Este es el beneficio que un directivo «que no ve el beneficio de digitalizarse» sí entiende: **no perder la autorización**. Un calendario determinista de vencimientos sirve como gancho de valor:
  - la vigencia de la constancia;
  - el 31 de enero, el 15 de febrero y mayo;
  - la opinión de cumplimiento.

  Lo calcula el código, no la IA, y vale más que cualquier función de IA.

### 1.3 Capacidades, tiempo, miedo y costo (encuestas globales)

- **[H]** Salesforce, *Nonprofit Trends Report*, 7.ª edición (2025). https://www.salesforce.com/en-us/wp-content/uploads/sites/4/2025/05/s-org-7th-edition-nonprofit-trends-report-2025.pdf
  - **Personal.** Es el reto operativo más extendido: dos tercios tienen problemas para contratar, retener y por agotamiento.
  - **Demanda.** Casi la mitad reporta más demanda, pero solo un tercio tiene más capacidad.
  - **Uso de IA.** El 55 % usa o pilotea IA (12 % el año anterior). Las organizaciones pequeñas tienen el uso más bajo (28 %) y la menor apertura (40 %).
  - **Barreras.** Entre las que no usan IA, más de la mitad «no sabe por dónde empezar». El 63 % se preocupa por la privacidad y la seguridad de los datos, un tercio dice que le faltan habilidades internas y una cuarta parte que le falta presupuesto.
  - **Seguridad.** Solo el 41 % capacita a su personal en ciberseguridad.
- **[H]** TechSoup y Tapp Network, *State of AI in Nonprofits 2025*: 1,321 respuestas, encuesta del 2.º y 3.er trimestre de 2024. https://message.techsoup.org/hubfs/Downloads/The%20State%20of%20AI%20in%20Nonprofits%202025.pdf
  - **Estrategia y política.** El 76 % no tiene estrategia de IA y el 80 % no tiene política de uso aceptable.
  - **Costo.** El 30 % lo ve como barrera, y el 50 % opera con menos de 500 mil dólares al año.
  - **Quién decide.** El 43 % depende de una sola persona para las decisiones de TI e IA.
  - **Qué quieren.** El 60 % tiene interés en ayuda para redactar solicitudes de fondos (*grant writing assistance*).
  - **Experiencia.** Solo el 6 % se considera experto.
- **[H]** Connect Humanity y TechSoup, más de 7,500 organizaciones (2023). El 78 % dice que la falta de acceso, herramientas o habilidades limita su servicio, y pocos sienten que su personal esté bien capacitado. https://www.alliancemagazine.org/blog/are-barriers-to-digital-technology-preventing-civil-society-organisations-from-achieving-their-potential/
- **[H]** LACNIC, OSC de Latinoamérica con un caso en México. Usan herramientas digitales sobre todo en comunicación, recaudación y gestión interna, y piden financiamiento específico para tecnología y capacitación. https://www.lacnic.net/innovaportal/file/7215/1/executive-summary-joaquin-herrero.pdf
- **[H]** El ICNL y Comecso señalan que muchas OSC mexicanas «no cuentan con herramientas mínimas» y que la falta de fondos para profesionalizarse crea un círculo vicioso. https://www.icnl.org/wp-content/uploads/our-work_ablanedo-estudio-final.pdf y https://www.comecso.com/wp-content/uploads/2018/12/Cuaderno_LFFAROSC_IBD.pdf

### 1.4 Por qué abandonan una herramienta (evidencia débil, sobre todo de practicantes)

- **[H][V]** Las causas que más se repiten:
  - el cambio se gestiona como algo secundario;
  - la rotación de personal: quien recibió la capacitación se va;
  - la doble captura: la misma información se escribe cuatro veces;
  - los sistemas sobrecargados de campos.

  Un caso pasó de 20 % a 85 % de adopción al reducir el sistema a un objeto y diez campos. Son fuentes de vendedores y consultores, no encuestas.
  - https://bloomerang.com/blog/your-roadmap-for-nonprofit-crm-success
  - https://salesforcebreak.com/2026/01/28/nonprofit-ui-usability-adoption/
  - https://www.sopact.com/guides/nonprofit-crm
- **[H]** La «barrera de articulación» (Jakob Nielsen, 2023). Las interfaces que dependen de escribir un *prompt* exigen saber redactar. Cerca de la mitad de la población de países ricos tiene baja alfabetización funcional, y en países de ingreso medio «es mucho peor». En 2025 propone patrones para compensarlo: constructores de *prompts*, parametrización y reescritura sugerida.
  - https://jakobnielsenphd.substack.com/p/prompt-driven-ai-ux-hurts-usability
  - https://jakobnielsenphd.substack.com/p/prompt-augmentation
- **[H]** En una comunidad de adultos mayores, del 0 al 12 % usaba la tecnología disponible aunque el 85 % quería aprender. Las causas: interfaces complejas, letra pequeña y falta de guía a la medida. https://arxiv.org/pdf/2504.08985
- **[O]** Para el directivo de SociAI, la caja de texto libre es la peor puerta: pide justo la habilidad que le falta. Las preguntas deben ser cerradas o con opciones, y el texto libre debe ser opcional, no el camino principal.

---

## 2. Patrones de producto y UX para usuarios no técnicos con IA

### 2.1 Marcos de diseño serios

- **[H]** Microsoft, *Guidelines for Human-AI Interaction*: 18 guías validadas con 49 diseñadores sobre 20 productos (CHI 2019), organizadas en cuatro momentos (al inicio, durante, cuando se equivoca y con el tiempo). https://www.microsoft.com/en-us/research/publication/guidelines-for-human-ai-interaction/ y https://www.microsoft.com/en-us/haxtoolkit/
  - Las más pertinentes para SociAI:
    - G1, decir qué puede hacer el sistema;
    - G2, decir qué tan bien lo hace;
    - G8, descartar con facilidad;
    - G9, corregir con facilidad;
    - G10, acotar el servicio cuando hay duda;
    - G11, decir por qué hizo lo que hizo;
    - G16, comunicar las consecuencias de una acción;
    - G17, dar controles globales.
  - La lista viene del artículo CHI 2019; la página del kit no la reproduce.
- **[H]** Google PAIR, *People + AI Guidebook*.
  - El capítulo «Explainability + Trust» propone **calibrar** la confianza: ni muy poca ni de más.
  - «Errors + Graceful Failure» pide definir qué es un error, de dónde viene y **ofrecer un camino para seguir**. El manejo de errores corrige el modelo mental de la persona.
  - https://pair.withgoogle.com/chapter/People%20+%20AI%20Guidebook%20-%20Explainability%20+%20Trust.pdf
  - https://pair.withgoogle.com/chapter/People%20+%20AI%20Guidebook%20-%20Errors%20+%20Graceful%20Failure.pdf
- **[H]** *The Shape of AI* (Emily Campbell), un catálogo de patrones: *identifiers* (dónde está la IA), *wayfinders*, *prompts*, *tuners* y *trust indicators*. https://jarango.com/2024/05/20/the-informed-life-with-emily-campbell/

### 2.2 «La persona cuenta y el sistema llena»: ejemplos reales

- **[H] Microsoft Business Central, Autofill (Copilot).** https://learn.microsoft.com/en-us/dynamics365/business-central/faqs-autofill
  - **Nada se guarda sin aceptar.** Sugiere valores para un grupo de campos, pero no los guarda: la persona revisa y acepta **cada campo**. Lo no aceptado se descarta al salir de la página.
  - **El valor aceptado cuenta como suyo.** Se guarda «como si usted hubiera escrito ese valor».
  - **La IA no siempre inventa el valor.** Muchas sugerencias vienen de mecanismos deterministas: el valor más frecuente, el más reciente o una selección entre opciones de catálogo. La IA solo elige qué mecanismo aplica y si la sugerencia es relevante.
  - **Corre con los permisos de la persona.**
- **[H] Business Central, página `PromptDialog`.** El patrón estándar de Copilot: la persona da la entrada, ve la propuesta y la conserva o la descarta. Las acciones del sistema son fijas: Generate, Regenerate, Attach, OK y Cancel. https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-page-type-promptdialog
- **[H] Business Central, Sales Order Agent.** Es la referencia más cercana a SociAI en un ERP. https://learn.microsoft.com/en-us/dynamics365/business-central/faqs-sales-order-taker-agent
  - **Su propio usuario.** Corre con un conjunto de permisos y un perfil de interfaz que limitan qué páginas, campos y acciones ve. Todo lo que hace lleva su ID de usuario en el historial.
  - **Pide a una persona** cuando hace falta: antes de todo correo de salida, cuando necesita datos para destrabarse y para aprobar cambios importantes. Avisa en el centro de rol.
  - **Línea de tiempo por tarea.** Muestra los pasos del agente y de las personas, y puede mostrar el razonamiento y las **citas** que llevaron a un valor.
  - **No crea datos maestros** (artículos, clientes), **no contabiliza** y **no calcula precios ni descuentos**: los calcula la lógica del ERP y lo que piden lo pasa a una persona.
  - **Batería de más de 70 casos** por categoría, incluidas «seguridad y límites» (no prometer descuentos) y «fuera de alcance» (lo que debe pasar a una persona).
- **[H][V] Ramp, gastos.** Tres niveles: sugerencia (el campo queda vacío hasta elegir), prellenado (requiere revisión) y autollenado (sin revisión, solo para comercios muy predecibles). La automatización **sube** si la persona acepta seguido durante dos semanas y **baja** si edita seguido. https://support.ramp.com/hc/en-us/articles/360042588454-Submitting-receipts-memos-and-accounting-for-your-Ramp-transactions
- **[H] TurboTax, foto del W-2.** Extrae los datos de una foto o un PDF. La comunidad de soporte recomienda revisar cada casilla, porque las fotos generan más errores en ciertas casillas. La foto no se guarda. https://ttlc.intuit.com/turbotax-support/en-us/help-article/import-export-data-files/import-enter-w-2/L55HzdeDr_US_en_US y https://ttlc.intuit.com/community/taxes/discussion/can-i-upload-w-2/00/3700082
- **[H] Encuestas conversacionales.** En un estudio de campo con unas 600 personas (Xiao et al., ACM TOCHI 2020), un chatbot que hacía preguntas abiertas obtuvo más participación y respuestas mejores que un formulario Qualtrics: más informativas, relevantes, específicas y claras. https://arxiv.org/abs/1905.10700
- **[H][V] Lemonade (Maya).** Hace una pregunta a la vez en forma de conversación y da el precio antes de pedir el correo. Las cifras de conversión que circulan vienen de vendedores de chatbots. https://getperspective.ai/blog/lemonade-case-study-conversational-ai-insurance
- **[H] Citas y confianza.** Mostrar citas aumenta significativamente la confianza en la respuesta de un LLM (AAAI-25). La confianza sube aunque la cita no se revise, así que también puede inflarla. https://arxiv.org/pdf/2501.01303
- **[H] MCP, *elicitation*.** Es el estándar para que un sistema le pida datos a una persona a mitad de una tarea.
  - **Formularios planos de tipos simples:** texto con formato (correo, fecha), número, sí/no y enumeraciones con nombre visible.
  - **Tres respuestas:** aceptar, rechazar y cancelar.
  - **Lo sensible no se pide por esta vía,** y la persona puede revisar su respuesta antes de enviarla.

  https://modelcontextprotocol.io/specification/2025-06-18/client/elicitation

### 2.3 Lo que esto dice para SociAI [O]

1. **Sugerencia por campo, con aceptar y descartar, y nada se guarda sin el clic.** Es el patrón de la industria (Business Central y MCP). El código ya tiene un solo punto de guardado para lo que propone la IA.
2. **Decir de dónde salió cada valor**, como hace Business Central con sus mecanismos. Ejemplos: «del acta, página 3», «de lo que usted dijo», «calculado». El código ya guarda el origen de cada dato; falta que la persona **lo vea** junto a la sugerencia.
3. **Preguntar con opciones y no con texto libre.** El estudio de Xiao respalda la conversación de una pregunta a la vez, pero la barrera de articulación pide que cada pregunta traiga botones u opciones (como la *elicitation* de MCP) y que el texto libre sea opcional.
4. **El error nunca bloquea: siempre ofrece un camino para seguir** (PAIR). Ejemplos: «No pude leer la página 2; súbala otra vez o escriba el dato aquí».
5. **No copiar el autollenado sin revisión de Ramp.** En SociAI todo dato de la institución tiene consecuencias legales. A lo más, la próxima sugerencia de un mismo campo puede venir preseleccionada si la persona aceptó muchas, pero nunca debe guardarse sola.
6. **Una línea de tiempo para lo que hizo la IA,** como la de Business Central: qué leyó, qué sugirió, quién aceptó. El código ya tiene la bitácora y el historial de cada dato; falta mostrarlos como relato.

---

## 3. Arquitectura de un ERP donde la IA actúa

### 3.1 Guías de los laboratorios

- **[H] Anthropic, «Building effective agents».** https://www.anthropic.com/engineering/building-effective-agents
  - **Flujos y agentes son cosas distintas.** Un *workflow* sigue caminos que el código predefine; un *agent* deja que el modelo decida sus pasos.
  - **La regla:** «la solución más simple posible y solo aumentar la complejidad cuando haga falta». Una sola llamada con recuperación y ejemplos muchas veces basta.
  - **Patrones de flujo:** encadenar *prompts* con revisiones del código entre pasos, **enrutar** (clasificar y despachar a una tarea especializada), paralelizar, orquestador con trabajadores, y evaluador con optimizador.
  - **La interfaz entre agente y computadora (ACI)** merece tanto cuidado como la interfaz humana. Hay que hacer las herramientas a prueba de errores (*poka-yoke*): cambiar los argumentos para que sea más difícil equivocarse.
- **[H] Anthropic, «Writing effective tools for agents».** https://www.anthropic.com/engineering/writing-tools-for-agents
  - **Pocas herramientas de alto impacto,** no una por cada *endpoint*: `search_contacts` mejor que `list_contacts`.
  - **Agrupar por prefijo** (*namespacing*).
  - **Devolver contexto con significado:** nombres legibles, no identificadores.
  - **Ahorrar tokens:** paginar y truncar.
  - **Errores que dicen cómo corregir.**
  - **Evaluar con tareas realistas de varios pasos** y leer las transcripciones.
- **[H] Anthropic, «Effective context engineering».** https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents
  - **El contexto es un recurso finito** con rendimientos decrecientes.
  - **El *prompt* va a la «altura correcta»:** ni lógica rígida ni vaguedad.
  - **Pocos ejemplos canónicos** en lugar de listas de casos.
  - **Recuperar justo a tiempo,** con referencias ligeras.
  - **Subagentes** que devuelven un resumen condensado.
- **[H] OpenAI, «A practical guide to building agents».** https://cdn.openai.com/business-guides-and-resources/a-practical-guide-to-building-agents.pdf
  - **Cuándo vale un agente:** cuando hay juicio complejo, reglas inmanejables o datos no estructurados; «si no, una solución determinista puede bastar».
  - **Primero exprimir un solo agente** antes de varios.
  - **Defensa por capas:** clasificador de relevancia, filtro de PII, salvaguardas de herramientas, reglas deterministas y validación de la salida.
  - **Cada herramienta lleva un riesgo** (bajo, medio o alto) según lectura o escritura, reversibilidad, permisos que requiere e impacto financiero. El riesgo dispara pausas o el paso a una persona.
  - **Dos disparadores para pasar a una persona:** pasar un umbral de fallas y una acción de alto riesgo.

### 3.2 Lo que hacen los ERP y CRM

- **[H] Salesforce Agentforce, «hybrid reasoning» y Agent Script (2025-2026).** Salesforce corrigió su modelo anterior, en el que el LLM decidía cada paso. https://architect.salesforce.com/docs/architect/fundamentals/guide/hybrid-reasoning-agentforce-builder-agent-script
  - **Qué fallaba:**
    - caminos inconsistentes ante entradas iguales;
    - costo y latencia (al menos tres ciclos de LLM por conversación simple);
    - huecos en la auditoría;
    - estado perdido (repetir pasos ya hechos).
  - **La regla nueva:** «si puede expresar una decisión como código, escríbala como lógica».
    - **Del código:** la navegación entre etapas, las validaciones, las matemáticas, traer datos y las condiciones.
    - **Del LLM:** entender la intención, redactar y manejar lo ambiguo.
  - **Las compuertas de seguridad son del código, no del modelo.** «Si deja que el LLM ponga la variable de la compuerta, reintroduce la variabilidad que la compuerta debía evitar». Una acción con `available when` falso **se quita de la lista** de herramientas del modelo: es una compuerta de plataforma, no una instrucción del *prompt*.
  - Los «Topics» se llaman ahora «subagents».
  - Ver también: https://developer.salesforce.com/blogs/2025/12/master-hybrid-reasoning-with-the-new-agent-script-recipes-sample-app
- **[H] Agentforce, Atlas y la Trust Layer.** Atlas evalúa la petición contra los temas disponibles y elige uno; las acciones respetan los permisos de quien pregunta. https://www.salesforce.com/agentforce/how-it-works/
- **[H] SAP Joule Studio.** Distingue *skills* de *agents*. Es casi el mismo corte que operador y agente en SociAI.
  - **Skills:** operaciones simples y repetitivas, con entradas y salidas predefinidas.
  - **Agents:** problemas de varios pasos.
  - **Supervisión humana** activa por defecto en las decisiones críticas.
  - https://www.sap.com/products/artificial-intelligence/joule-studio.html
  - https://learning.sap.com/courses/introducing-joule-studio-in-sap-build/leveraging-custom-joule-agents-for-complex-multi-step-business-workflows
- **[H] ServiceNow.** La supervisión es una opción **por herramienta**: un modo supervisado pausa hasta que una persona aprueba. Un ejecutivo de ServiceNow recomienda que las lecturas sean autónomas y todas las escrituras supervisadas [V, fuente patrocinada].
  - https://www.techtarget.com/ai/news/366618614/ServiceNow-intros-AI-agent-studio-and-orchestrator
  - https://nojitter.com/ai-automation/servicenow-on-orchestrating-a-symphony-of-ai-agents
- **[H] Odoo 19.** Un agente son *topics* (instrucciones más herramientas) y *sources* (documentos indexados).
  - **Sin topics, solo informa:** no puede cambiar la base.
  - **«Restrict to Sources»:** lo limita a sus documentos.
  - **Sin confirmación humana documentada** antes de escribir.
  - https://www.odoo.com/documentation/19.0/applications/productivity/ai/agents.html
- **[H] Microsoft Business Central.** Ver §2.2: permisos y perfil propios del agente, aprobaciones, la línea de tiempo y la batería por categorías.

### 3.3 Herramientas generadas desde el esquema, la capa semántica y MCP

- **[H] Muchas herramientas degradan la elección.** El artículo RAG-MCP mide que, con muchas herramientas en el *prompt*, la elección correcta cae al 13.62 %. Recuperar solo las relevantes la sube al 43.13 % y reduce los tokens a menos de la mitad. https://arxiv.org/html/2505.03275v1
- **[H] Las herramientas CRUD generadas no son las que conviene dar al modelo.** Una propuesta de GitLab sugiere convertir las herramientas CRUD en un conjunto reducido con entradas más amigables para un LLM. https://gitlab.com/gitlab-org/gitlab/-/issues/561293
- **[H][V] Capa semántica.**
  - **data.world:** GPT-4 contestando sobre SQL directo acertó el 16 %; sobre un grafo de conocimiento del mismo esquema, el 54 %. https://iianalytics.com/community/blog/knowledge-graphs-improve-accuracy-of-large-language-models
  - **dbt:** dice que, con una capa semántica bien modelada, las preguntas que cubre se acercan al 100 %. Benchmark pequeño (11 preguntas) y vendedores interesados. https://docs.getdbt.com/blog/semantic-layer-vs-text-to-sql-2026
- **[H] MCP, especificación de herramientas.** https://modelcontextprotocol.io/specification/2025-06-18/server/tools
  - **Persona en el ciclo (*SHOULD*).** Mostrar qué herramientas ve el modelo, indicar cuándo se invocan y pedir confirmación para las operaciones.
  - **Revisar entradas y salidas.** Validar las entradas, mostrar a la persona la entrada de la herramienta antes de llamarla (para evitar la exfiltración), validar los resultados contra su `outputSchema` y registrar el uso para auditoría.
  - **Anotaciones.** `readOnlyHint` y `destructiveHint` son pistas «no confiables» si el servidor no lo es. Por defecto, una herramienta sin anotar se trata como destructiva. https://www.speakeasy.com/docs/mcp/build/gram-functions/tool-annotations

### 3.4 Lo que esto dice para SociAI [O]

1. **Lo determinista al código, lo ambiguo a la IA.** Es lo que la industria aprendió a golpes: Anthropic, OpenAI y SAP lo dicen, y Salesforce rehízo Agentforce hacia ahí.
2. **Enrutar** (el patrón «routing» de Anthropic y los subagentes de Salesforce). La lección de Salesforce: la etapa, los permisos y lo que ya pasó (el «gate») los pone el código, nunca el modelo. Una herramienta que no aplica en este momento **no se le ofrece** al modelo; no basta con decirle en el *prompt* que no la use.
3. **Cada herramienta lleva una tabla de riesgo** (estilo OpenAI): lectura o escritura, reversible o no, protegida o no.
   - Lectura: autónoma.
   - Escritura: siempre como sugerencia.
   - Protegido: nunca.

   Hay que dejarlo escrito en la descripción de cada herramienta.
4. **Una sola descripción de los campos es la «capa semántica» de SociAI.** Generar herramientas desde ella es correcto (qué acepta cada campo, sus opciones, su nombre en pantalla). Pero al modelo se le dan **pocas herramientas por intención**, no una por campo o por tabla. Ejemplos: «estado de llenado», «buscar en el expediente», «proponer valores para una ventana».
5. **La batería por categorías, como la de Business Central:** caso normal, datos incompletos, ambigüedad, seguridad (lo que nunca debe prometer o guardar), **fuera de alcance** (lo que debe pasar a una persona o decir «no puedo») y varios turnos. Conviene etiquetar cada caso con su categoría para ver huecos.
6. **MCP no hace falta dentro de la app,** que es de escritorio, con un solo proceso Rust. Sí conviene copiar sus reglas: `outputSchema` validado, la persona ve la entrada de la herramienta en lo que escribe, la bitácora de uso y *elicitation* como modelo de «preguntar a la persona».

---

## 4. Privacidad con LLM en la nube

### 4.1 Lo que exige la ley en México (LFPDPPP de 2025)

- **[H]** La nueva LFPDPPP se publicó en el DOF el 20 de marzo de 2025 y entró en vigor el 21; abrogó la ley de 2010. El texto vigente tiene una reforma del 14 de noviembre de 2025. Las funciones del INAI pasaron a la **Secretaría Anticorrupción y Buen Gobierno**. https://www.diputados.gob.mx/LeyesBiblio/pdf/LFPDPPP.pdf (texto vigente) y https://www.garrigues.com/es_ES/noticia/mexico-nueva-ley-federal-proteccion-datos-personales-posesion-particulares-introduce
- **[H]** Lo que dice el texto vigente (artículos citados de la ley):
  - **Art. 2 fr. VI.** Son datos sensibles, entre otros, los que revelan origen racial o étnico, **estado de salud presente o futuro**, información genética, creencias religiosas, opiniones políticas y preferencia sexual.
  - **Art. 2 fr. IX.** La **disociación** es el procedimiento por el que los datos no pueden asociarse a la persona ni permiten identificarla «por su estructura, contenido o grado de desagregación».
  - **Art. 8.** Los datos sensibles requieren **consentimiento expreso y por escrito**, con firma autógrafa, firma electrónica u otro mecanismo de autenticación. No pueden crearse bases con datos sensibles sin justificarlas por finalidades legítimas y concretas.
  - **Art. 9 fr. III.** No se requiere consentimiento si los datos se someten **antes** a disociación.
  - **Art. 12 (minimización).** El tratamiento debe ser «necesario, adecuado y relevante» para las finalidades. Con los datos sensibles, el periodo de tratamiento debe ser el «mínimo indispensable».
  - **Art. 18.** Medidas de seguridad administrativas, técnicas y físicas, según el riesgo y la sensibilidad.
  - **Art. 19.** Avisar de inmediato a la persona de una vulneración que afecte de forma significativa sus derechos.
  - **Art. 2 fr. XII y XX.** La **persona encargada** trata datos por cuenta del responsable. Comunicarle datos **no es una «transferencia»**, que por definición es hacia alguien distinto del titular, del responsable y del encargado.
  - **Art. 26 fr. II.** Derecho de oposición al tratamiento **automatizado sin intervención humana** que produzca efectos jurídicos o afecte de manera significativa. Por ejemplo, el que analiza o predice la salud o la situación económica.
  - **Arts. 59 y 64.** Con datos sensibles, las multas pueden aumentar hasta el doble y las penas se duplican.
- **[H]** El Reglamento de 2011, de la ley anterior, pedía:
  - **Art. 51:** un contrato entre el responsable y el encargado.
  - **Art. 52:** que el proveedor de **cómputo en la nube** tenga políticas de protección, transparente sus subcontrataciones, no reclame propiedad de los datos, guarde confidencialidad, borre al terminar el servicio y permita limitar el tratamiento.

  No encontré un reglamento nuevo publicado. El transitorio Décimo Segundo daba 90 días para armonizar los reglamentos. **Hay que verificar con un abogado qué reglamento rige hoy.** https://mley.mx/Reg_LFPDPPP/articulo/52/
- **[O] Lectura para SociAI (no es asesoría legal):**
  - **El proveedor de IA sería un encargado, no un receptor de transferencias.** Eso exige condiciones de servicio que cumplan con lo del art. 52: no entrenar con los datos, borrarlos y confidencialidad.
  - **Lo que sale ya disociado** (conteos, agregados de 3 o más) queda fuera del consentimiento por el art. 9 fr. III. Es el mejor respaldo legal de «cuántos, nunca quiénes».
  - **Si algún día se guardan expedientes de beneficiarios con su salud,** el programa debe ayudar a recabar el consentimiento por escrito (art. 8) y a escribir el aviso de privacidad. Conviene decidirlo antes de hacer ese módulo.

### 4.2 Los proveedores: un hallazgo importante

- **[H] Gemini API, términos. Los servicios no pagados** (el nivel gratuito y AI Studio):
  - Google usa lo que se envía y lo que el modelo responde «para proveer, mejorar y desarrollar» sus productos y su aprendizaje automático;
  - **personas revisoras** pueden leer las entradas y salidas, desvinculadas de la cuenta;
  - los términos dicen literalmente: *«Do not submit sensitive, confidential, or personal information to the Unpaid Services.»*

  **Los servicios pagados** no se usan para mejorar productos; se registran por un tiempo limitado solo para detectar abusos. https://ai.google.dev/gemini-api/terms
- **[H] Anthropic API.** Borra entradas y salidas en 30 días por defecto (salvo excepciones) y no entrena con datos de la API sin permiso explícito. Algunos clientes empresariales tienen retención cero. https://privacy.claude.com/en/articles/7996866-how-long-do-you-store-my-data
- **[O] Riesgo concreto.** Las pruebas de SociAI usaron el plan gratuito de Gemini (15 solicitudes por minuto, 500 al día). Con el nivel gratuito, **nada que sea personal o confidencial** debería salir hacia Gemini. Eso incluye los nombres del patronato o de la directora y el texto del expediente, aunque sea de la institución. Para producción con instituciones reales, propongo dos condiciones:
  - exigir una llave de un proyecto con facturación (nivel pagado), o Anthropic;
  - o que el código se niegue a mandar a un proveedor gratuito lo que no sea público.

  El líder decide en un ADR. Es el hallazgo más urgente de este reporte.

### 4.3 Patrones técnicos: seudonimizar, tapar y restaurar

- **[H] Salesforce Einstein Trust Layer.** https://trailhead.salesforce.com/es/content/learn/modules/llm-data-masking-in-the-einstein-trust-layer/explore-llm-data-masking y https://developer.salesforce.com/docs/einstein/genai/guide/data-masking.html
  - **Cómo trabaja.** Detecta con patrones y aprendizaje automático, reemplaza por marcadores (`<Person_0>`) y **restaura** los valores en la respuesta. Deja registro en una bitácora.
  - **Falso positivo documentado.** Un número de pedido de 10 dígitos se tapó por parecer teléfono, y la respuesta empeoró.
  - **Desactivado para los agentes.** La misma página dice que la máscara de datos para el LLM está **desactivada para los agentes**.
- **[H] Microsoft Presidio.** Detecta y anonimiza; admite español con un modelo de spaCy (`es_core_news_md`/`lg`). Sin ese modelo, el español se confunde: «Yo» salía como PERSON. El propio proyecto advierte que no garantiza encontrar todo y que hacen falta protecciones adicionales.
  - https://python.langchain.com/docs/guides/privacy/presidio_data_anonymization/multi_language
  - https://microsoft.github.io/presidio/faq/
  - https://www.thoughtworks.com/radar/languages-and-frameworks/presidio
- **[H] LLM Guard.** Su escáner `Anonymize` guarda los valores reales en una bóveda (`Vault`); `Deanonymize`, aplicado a la salida, cambia los marcadores por los originales. Debe usarse la misma bóveda de la anonimización. https://langfuse.com/docs/security/getting-started
- **[H] Efecto en la calidad.**
  - **Seudonimizar daña más el resumen que la clasificación.** Las tareas que dependen de entidades sufren más (arXiv 2306.05561). https://arxiv.org/pdf/2306.05561
  - **Los sustitutos realistas y del mismo tipo** (otro nombre en lugar de `[NOMBRE]`) conservan mejor la utilidad, incluso en preguntas y respuestas con agentes (arXiv 2603.17217, 2026). Es la afirmación de los autores; no revisé sus tablas. https://arxiv.org/abs/2603.17217
  - **Hay un marco general para LLM en la nube** con detección, sustitución y restauración (arXiv 2502.15233). https://arxiv.org/abs/2502.15233
- **[H] NER local en español.**
  - **MEDDOCAN** (desidentificación clínica en español): los mejores sistemas logran un F1 de unos 0.97 en su dominio. https://portal.odesia.uned.es/en/node/235 y https://arxiv.org/pdf/2007.01030
  - **Pero no generaliza:** un modelo entrenado en MEDDOCAN bajó a 38.89 de F1 en informes de radiología. https://biomedcentral-jbiomedsem.publicaciones.saludcastillayleon.es/articles/10.1186/s13326-021-00236-2/tables/8
  - **GLiNER multi PII** (`urchade/gliner_multi_pii-v1`, Apache 2.0): cubre español y hay exportaciones ONNX de la comunidad (unos 278 millones de parámetros). **GLiNER2-PII** reconoce 42 tipos de entidad. https://www.promptlayer.com/models/glinermultipii-v1, https://huggingface.co/lBroth/nullpii y https://arxiv.org/pdf/2605.09973

### 4.4 Lo que esto dice para SociAI [O]

1. **Clasificar por necesidad antes de tapar.** La defensa más fuerte no es un tapado perfecto; es que el dato **no esté en el contexto** porque la tarea no lo necesita (art. 12, minimización). Es lo que dice «cuántos, nunca quiénes». Conviene una tabla única con cuatro clases:
   - **público o de la institución,** que va (y solo a un proveedor pagado);
   - **propio protegido** (RFC, CLABE, folios), que solo leen las reglas;
   - **personal de terceros,** que nunca va;
   - **sensible,** que nunca va y solo como agregado de 3 o más.
2. **Tapar con sustitutos consistentes solo donde la tarea necesita al personaje.** Por ejemplo, una persona dictó un texto con nombres y la IA debe redactar. Los marcadores tipo `[NOMBRE_1]` restaurados por el código sirven para operadores, pero Salesforce los apagó en agentes; hay que medir antes de usarlos en el Asistente.
3. **NER local como segunda capa detrás de las reglas,** no como única. El hallazgo de MEDDOCAN (0.97 en su dominio, 0.39 fuera) dice que se mide con los propios documentos de SociAI. GLiNER multi PII es una alternativa con licencia Apache que vale la pena medir.
4. **Medir los falsos positivos además de los negativos.** El caso del número de pedido de Salesforce es justo el de los folios y números de oficio de SociAI. Un tapado de más degrada la respuesta y confunde a la persona.
5. **La oposición al tratamiento automatizado** (art. 26 fr. II) refuerza «nada se guarda sin una persona». Ninguna decisión sobre una persona (beneficiario o personal) debe tomarla la IA sola.

---

## 5. Resumen de fuentes por calidad

| Calidad | Fuentes |
|---|---|
| Primaria y sólida | La LFPDPPP vigente; los términos de Gemini y la privacidad de Anthropic; Microsoft Learn (Business Central); la especificación de MCP; Anthropic y OpenAI (guías); la arquitectura de Salesforce; Cemefi (datos del registro y guía 2026); INEGI; los artículos arXiv citados |
| Encuesta con muestra declarada | Salesforce, 7.ª edición (2025); TechSoup y Tapp (1,321 respuestas); Connect Humanity (7,500); Xiao et al. (unas 600 personas) |
| Débil o de vendedor [V] | Lemonade; Ramp (cifras de marketing); ServiceNow (artículo patrocinado); dbt, AtScale y Cube (capa semántica); Bloomerang y consultores (abandono de CRM); cifras de revocaciones de 2023 y 2024 (Forbes) |
| No encontrado | Estudios de Alternativas y Capacidades o de Cemefi sobre adopción tecnológica de OSC mexicanas; un reglamento nuevo de la LFPDPPP; la lista oficial de las 18 guías en la página del kit HAX (está en el artículo CHI 2019) |
