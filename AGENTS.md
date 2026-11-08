# AGENTS.md — CRM Inmobiliario Local-First con Grafo CNA y EGA

## 🎯 Visión del Producto
CRM inmobiliario **local-first** que combina análisis de redes sociales de clientes (CNA) con análisis de gravedad económica de zonas (EGA) para predecir efectos dominó de compra y detectar zonas de auge con "ventaja del grafo local". **Ningún dato sale del dispositivo del usuario.**

## 🏗️ Arquitectura Técnica Confirmada

### Frontend
- **Tauri 2.x** (Rust) — shell nativa ligera y con capacidades explícitas
- **SvelteKit 5.x** — UI reactiva con runes (`$state`, `$derived`, `$effect`)
- **Cytoscape.js 3.x** — visualización y manipulación del grafo CNA
- **MapLibre GL JS 4.x** — mapas y capas de calor EGA mediante WebGL/WebGPU cuando esté disponible
- **TanStack Query 5.x + Dexie.js 4.x** — caché asíncrona y acceso tipado a IndexedDB para el estado offline-first
- **TypeScript estricto** (`strict: true`) y **Tailwind CSS 4.x**

### Backend (Rust sidecar)
- **Lógica de negocio**: Rust (Tauri commands)
- **Transaccional**: SQLite + `sqlite-vec` (vectores); usar `sqlx` o `rusqlite` según las necesidades de compilación y extensiones, evitando mantener dos capas de acceso sin una necesidad concreta
- **Analítica OLAP**: DuckDB embebido
- **Grafo CNA**: `petgraph` + persistencia en SQLite
- **ML**: ONNX Runtime (modelos entrenados fuera, inferencia local)
- **Geoespacial**: crates del ecosistema `georust`, incluyendo H3 y PROJ cuando sean necesarios
- **Cifrado y claves**: SQLCipher para cifrado de base de datos; Argon2id para derivar claves. Usar primitivas criptográficas de bibliotecas mantenidas y nunca simular cifrado o hashing.

## 📁 Estructura de Carpetas Esperada

```text
crm-inmobiliario/
├── src-tauri/                 # Backend Rust
│   ├── src/
│   │   ├── commands/          # Tauri commands (API pública al frontend)
│   │   ├── db/                # SQLite + DuckDB setup
│   │   ├── graph/             # Motor CNA (petgraph)
│   │   ├── ega/               # Economic Gravity Analysis
│   │   ├── matching/          # Matching predictivo (ONNX)
│   │   ├── cascade/           # Motor de propagación en cascada
│   │   ├── osint/             # Enriquecimiento con fuentes públicas
│   │   ├── crypto/            # Auditoría criptográfica (hashes, firmas)
│   │   └── models/            # Estructuras de datos compartidas
│   └── migrations/            # SQL migrations
├── src/                       # Frontend SvelteKit
│   ├── lib/
│   │   ├── components/
│   │   │   ├── graph/         # Visualizaciones Cytoscape
│   │   │   ├── map/           # MapLibre + capas EGA
│   │   │   └── ui/            # Componentes base
│   │   ├── stores/            # TanStack Query + stores Svelte
│   │   └── services/          # Wrappers de Tauri commands
│   └── routes/
└── AGENTS.md                  # ESTE ARCHIVO
```

## 🧩 Módulos y Sus Interfaces Esperadas

### Módulo 1: Clientes / Leads
**Entidad `Client`**:
```rust
pub struct Client {
    pub id: Uuid,
    pub nombre: String,
    pub apellido_paterno: String,
    pub apellido_materno: String,
    pub curp: Option<String>,           // hash + salt, nunca en claro
    pub edad: u8,
    pub presupuesto: Decimal,
    pub tipo_credito: TipoCredito,      // Infonavit, Bancario, CASH, etc.
    pub empresa: Option<String>,
    pub ingreso_mensual: Decimal,
    pub edad_laboral: u8,
    pub created_at: DateTime<Utc>,
    pub consentimientos: Consentimientos, // RGPD/LFPDPPP
}
```

**Reglas de negocio**:
- CURP y datos sensibles NUNCA se almacenan en claro. Usar SQLCipher o cifrado autenticado a nivel de aplicación con clave derivada de contraseña maestra mediante Argon2id. Si se requiere comparar CURP sin recuperarlo, usar un HMAC con clave secreta en vez de un hash simple con salt.
- Cada modificación genera un evento auditable en tabla `audit_log`.
- No implementar criptografía casera ni reemplazar una protección requerida por una simulación o un marcador de posición.

### Módulo 2: Propiedades
**Entidad `Property`**:
```rust
pub struct Property {
    pub id: Uuid,
    pub costo: Decimal,
    pub recamaras: u8,
    pub banos: f32,
    pub niveles: u8,
    pub coordenadas: (f64, f64),        // WGS84
    pub poligono: Option<Vec<(f64, f64)>>, // para terrenos
    pub documentos: Vec<Documento>,     // hashes + rutas locales
    pub embedding: Vec<f32>,            // 384-dim para matching
}
```

### Módulo 3: CNA (Client Network Analysis)
**Entidad `GraphEdge`**:
```rust
pub struct GraphEdge {
    pub source: Uuid,      // client_id
    pub target: Uuid,      // client_id
    pub weight: f32,       // 0.0 - 1.0 (fuerza de relación)
    pub signals: Vec<Signal>, // evidencias que justifican el edge
    pub created_at: DateTime<Utc>,
}

pub enum Signal {
    MismoApellido { score: f32 },
    MismoEmpleador { empresa: String },
    MismaZonaVisita { zona_id: Uuid },
    InteraccionConfirmada { tipo: TipoInteraccion },
    SimilitudPerfil { cosine: f32 },
}
```

**Algoritmos requeridos**:
- `detect_communities()` → Louvain o Leiden
- `compute_pagerank()` → PageRank personalizado
- `find_structural_holes()` → detección de brokers (Burt's constraint)
- `simulate_cascade(client_id, event)` → ecuación de propagación

**Ecuación de propagación (implementar tal cual)**:
```
P(Cliente_i compra en t+Δt) = α·Σ_j [ W_ij · I_j(t) · e^(-λ·d_ij) ] + β·GeoScore(Zona) + γ·TemporalDecay(Δt)
```
- `α, β, γ` son aprendidos por ML (inicialmente 0.5, 0.3, 0.2)
- `λ = 0.5` (decaimiento por defecto)
- `I_j`: compra=1.0, visita=0.3, búsqueda=0.1

### Módulo 4: EGA (Economic Gravity Analysis)
**Entidad `POI`**:
```rust
pub struct POI {
    pub id: Uuid,
    pub tipo: TipoPOI,           // Escuela, Hospital, Metro, etc.
    pub masa: f32,               // tamaño/empleados/capacidad
    pub coordenadas: (f64, f64),
    pub fecha_apertura: Option<Date>,
    pub crecimiento_anual: f32,  // % expansión
}
```

**Tabla de pesos por defecto** (calibrables):
| Tipo | Masa | Radio (km) | Peso |
|------|------|------------|------|
| Escuela | 1000 | 2.0 | 0.95 |
| Hospital | 5000 | 5.0 | 0.85 |
| Metro | 800 | 0.8 | 0.92 |
| Centro Comercial | 2000 | 3.0 | 0.90 |
| Parque Industrial | 10000 | 5.0 | 0.88 |

**Fórmula** (implementar tal cual):
```
Atracción_POI→Zona = ( Masa × Peso_Categoría ) / ( Distancia^α × Decay )
ScoreFinal = ScoreBase × (1 + 0.3·DensidadCluster) × (1 + 0.5·ActividadCluster)
```
- `α = 1.8` (exponente de distancia, calibrable)

### Módulo 5: Matching Predictivo
**Modelo híbrido**:
1. **Content-based**: embeddings de propiedades (ONNX, modelo all-MiniLM-L6-v2)
2. **Collaborative**: propagación sobre el grafo CNA
3. **Geoespacial**: score EGA de la zona

**Score final**:
```
MatchScore(client, property) = 0.4·cosine(embedding_client, embedding_property)
                             + 0.3·cascade_score(client, zona_property)
                             + 0.3·ega_score(zona_property)
```

## 🔐 Convenciones de Código

### Rust
- Usar `thiserror` para errores, `serde` para serialización.
- Todos los `Tauri commands` deben validar inputs y devolver `Result<T, AppError>`.
- Nombres de funciones: `snake_case`. Tipos: `PascalCase`.
- Documentación pública con `///` en español.
- Tests unitarios en `#[cfg(test)] mod tests` dentro de cada módulo.

### Svelte / TypeScript
- TypeScript estricto (`"strict": true`).
- Componentes Svelte 5 con runes (`$state`, `$derived`, `$effect`).
- Stores con TanStack Query para datos remotos (Tauri commands).
- Estilos con Tailwind CSS 4.

### SQL
- Migraciones numeradas: `001_initial_schema.sql`, `002_add_audit_log.sql`.
- Usar UUIDs v7 (ordenables por tiempo).
- Índices en columnas de filtro frecuente.

## 🧪 Criterios de Aceptación por Módulo

### CNA
- [ ] Detectar comunidades con >85% de precisión en dataset sintético
- [ ] Identificar structural holes en <500ms para grafo de 10k nodos
- [ ] Simulación de cascada devuelve probabilidades normalizadas [0,1]

### EGA
- [ ] Calcular score de 1000 celdas × 500 POIs en <2s
- [ ] Capa de calor renderiza en MapLibre sin lag
- [ ] Score se actualiza incrementalmente al añadir POI

### Matching
- [ ] Top-10 propiedades recomendadas por cliente en <300ms
- [ ] Recall@10 > 0.6 en dataset de validación
- [ ] Explicabilidad: cada match muestra contribución de cada componente

## 🚦 Fases de Desarrollo (Prioridad)

### Fase 1 — MVP (4 semanas)
1. Schema SQLite + migraciones
2. CRUD Clientes y Propiedades (Tauri commands + Svelte UI)
3. CNA básico: edges manuales + detección de comunidades
4. Visualización del grafo con Cytoscape

### Fase 2 — Inteligencia Local (4 semanas)
1. Ingesta de POIs desde OpenStreetMap (Overpass API)
2. Motor EGA + capa de calor en MapLibre
3. Matching predictivo básico (content-based con embeddings)
4. Dashboard de zonas de auge con validación del grafo

### Fase 3 — Propagación y Simulación (3 semanas)
1. Motor de cascada completo
2. Simulador contrafactual ("¿y si vendo a X?")
3. Detección de "Pacientes Cero" y "Agujeros Estructurales"
4. Aprendizaje de pesos α, β, γ por retroalimentación

### Fase 4 — Hardening (3 semanas)
1. Cifrado de datos sensibles (SQLCipher + clave maestra)
2. Sistema de backup cifrado
3. Auditoría criptográfica de enriquecimientos OSINT
4. Tests de integración y benchmarks de rendimiento

## ⚠️ Reglas Inviolables

1. **Privacidad por diseño**: ningún dato sale del dispositivo sin acción explícita del usuario.
2. **Consentimiento auditable**: cada dato OSINT lleva hash de fuente + timestamp + consentimiento.
3. **Offline-first**: el sistema funciona 100% sin internet.
4. **Rendimiento nativo**: cualquier operación de UI debe responder en <100ms.
5. **Explicabilidad**: toda predicción muestra las señales que la justifican.

## 📚 Contexto para el Agente

Cuando generes código:
- Prefiere **composición sobre herencia**.
- Usa **tipos fuertes** (enums, newtypes) sobre strings sueltos.
- **Manejo de errores explícito**: nunca `unwrap()` en código de producción.
- **Inmutabilidad por defecto**: `&T` sobre `&mut T` cuando sea posible.
- **Tests primero**: escribe el test antes de la implementación cuando sea viable.
- **Documenta el "por qué"**, no el "qué".
- En comandos Tauri, valida entradas, documenta en español y devuelve errores tipados (`Result<T, AppError>`); no ocultes fallos con valores por defecto.
- En componentes Svelte, usa runes de Svelte 5, props tipadas y los servicios/queries existentes.
- Para CNA, aprovecha `petgraph` y prueba los objetivos de rendimiento con grafos representativos de 10k nodos.
- Para fórmulas CNA, EGA y matching, conserva exactamente las ecuaciones y coeficientes especificados; agrega pruebas numéricas para casos límite y documenta cualquier transformación posterior.
- La ecuación de cascada sin normalización puede producir valores fuera de `[0,1]`. No afirmes que su resultado está normalizado ni cambies silenciosamente la fórmula: resuelve por separado la calibración/normalización requerida y deja explícito su efecto en la explicabilidad.
- Si dos requisitos entran en conflicto o una dependencia/funcionalidad de seguridad no está disponible, explica el conflicto y pide una decisión o propone una alternativa segura; no la suplantes con una simulación.
- No envíes datos de clientes, relaciones ni información sensible a servicios externos. La red solo se usa con acción explícita para fuentes públicas, sin incluir datos privados del CRM.
