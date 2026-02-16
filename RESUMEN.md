# Resumen del Proyecto - gitzy

## Descripcion
**gitzy** es un cliente Git TUI (Terminal User Interface) minimalista construido en Rust con soporte para generacion de mensajes de commit con AI.

## Estado Actual
- **Version**: 0.1.0
- **Licencia**: MIT
- **Autor**: Jesus Campos (@yisas97)
- **Repositorio**: https://github.com/yisas97/gitzy

## Renombrado del Proyecto
El proyecto fue renombrado de **guit** a **gitzy**. Para completar el renombrado en GitHub:
1. Ir a Settings → General → Repository name
2. Cambiar "guit" a "gitzy"
3. GitHub redirigira automaticamente las URLs antiguas

## Tecnologias
- **Lenguaje**: Rust (Edition 2021)
- **Framework TUI**: ratatui 0.30.0
- **Terminal**: crossterm 0.29.0

## Estructura del Proyecto
```
gitzy/
├── src/
│   ├── main.rs      # Punto de entrada y event loop
│   ├── app.rs       # Logica de estado de la aplicacion
│   ├── git.rs       # Interfaz con comandos Git
│   ├── ui.rs        # Renderizado de la interfaz TUI
│   └── ai/          # Modulo de proveedores AI
│       ├── mod.rs       # Enum AiProvider + AiConfig
│       ├── claude.rs    # Integracion con Claude CLI
│       ├── ollama.rs    # Integracion con Ollama
│       ├── openai.rs    # Integracion con OpenAI via aichat
│       └── gemini.rs    # Integracion con Gemini via aichat
├── .github/
│   └── workflows/
│       └── release.yml  # CI/CD para releases multiplataforma
├── Cargo.toml       # Configuracion del proyecto
├── README.md        # Documentacion en ingles
├── README.es.md     # Documentacion en espanol
└── CONTRIBUTING.md  # Guia de contribucion
```

## Funcionalidades Principales
- Navegacion estilo Vim (j/k para mover, h/l para paneles)
- Vista de cambios en tiempo real (staged/unstaged)
- Diff integrado con resaltado de diferencias
- Gestion de stage (individual o masivo)
- Commits interactivos con editor integrado
- **Multi-proveedor AI** - Soporte para Claude, Ollama, OpenAI y Gemini
- Historial de commits (git log)
- Gestion de ramas (cambiar, crear)
- **Push/Pull/Fetch** - Operaciones remotas completas
- **Panel de Remotes** - Gestionar remotes (agregar, editar URL, eliminar)
- **Indicador de sync** - Muestra commits ahead/behind en el header
- **Ramas locales y remotas** - Ver y cambiar entre ramas locales y del remoto
- **Push de ramas** - Enviar ramas locales al servidor remoto

## Proveedores de AI
gitzy soporta multiples proveedores para la generacion de mensajes de commit:

| Proveedor | CLI | Instalacion |
|-----------|-----|-------------|
| Claude (default) | `claude` | `npm i -g @anthropic-ai/claude-code` |
| Ollama | `ollama` | https://ollama.ai |
| OpenAI | `aichat` | `cargo install aichat` + `OPENAI_API_KEY` |
| Gemini | `gemini` | `npm i -g @anthropic-ai/gemini-cli` o similar |

### Variables de Entorno
- `GITZY_AI_PROVIDER`: `claude` (default), `ollama`, `openai`, `gemini`
- `GITZY_OLLAMA_MODEL`: modelo de Ollama (default: `llama3.2`)
- `GITZY_OPENAI_MODEL`: modelo de OpenAI (default: `gpt-4o-mini`)
- `GITZY_GEMINI_MODEL`: modelo de Gemini (default: `gemini-1.5-flash`)

## Atajos de Teclado
| Tecla | Accion |
|-------|--------|
| `j/k` | Navegar arriba/abajo |
| `h/l` | Cambiar panel |
| `Space` | Stage/Unstage archivo |
| `c` | Abrir commit |
| `Tab` | Generar mensaje con AI (en commit) |
| `i` | Cambiar proveedor AI (en commit, solo si mensaje vacio) |
| `g` | Ver historial (log) |
| `b` | Gestionar ramas |
| `s` | Panel de remotes |
| `p` | Push |
| `P` | Pull |
| `f` | Fetch |
| `r` | Refrescar |
| `q` | Salir |

## Plataformas Soportadas
- Windows x64
- Linux x64
- macOS Intel (x64)
- macOS Apple Silicon (arm64)

## Comandos de Desarrollo
```bash
cargo run          # Ejecutar en desarrollo
cargo build --release  # Compilar optimizado
cargo test         # Ejecutar tests
```

## Publicacion
El proyecto esta preparado para publicarse en **crates.io** con el nombre `gitzy`.

## Cambios Recientes

### Mejoras en Commit y AI (Enero 2026)
- **Mejor manejo de errores en commit**: Ahora muestra el mensaje de error real de git (stdout o stderr)
- **Mensajes de commit mas cortos**: Prompt optimizado para generar mensajes de max 50 caracteres
- **Fix mensaje post-commit**: El mensaje "Commit: ..." ya no es sobrescrito por "Refrescado"
- **Limpieza de respuestas AI**: Trunca a 50 chars y solo toma la primera linea valida

---
*Ultima actualizacion: Enero 2026 - Mejoras en commits y generacion AI*
