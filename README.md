# guit

> Cliente Git TUI (Terminal User Interface) minimalista con soporte para AI

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org)

**guit** es una interfaz de terminal intuitiva y ligera para Git que te permite gestionar tus repositorios sin salir de la línea de comandos. Navega por cambios, stage archivos, visualiza diffs, administra ramas y crea commits, todo desde una interfaz elegante basada en texto.

## Características

- **Navegación intuitiva**: Usa atajos de teclado estilo Vim para moverte rápidamente
- **Vista de cambios en tiempo real**: Visualiza archivos modificados, staged y unstaged
- **Diff integrado**: Panel lateral con resaltado de diferencias
- **Gestión de stage**: Stage/unstage archivos individuales o todos a la vez
- **Commits interactivos**: Escribe mensajes de commit directamente en la interfaz
- **Soporte para AI**: Genera mensajes de commit automáticamente (Tab en modo commit)
- **Historial de commits**: Explora el log de Git sin salir de la aplicación
- **Gestión de ramas**: Cambia entre ramas o crea nuevas desde la interfaz
- **Ligero y rápido**: Construido en Rust con dependencias mínimas

## Instalación

### Descargar binarios precompilados

Descarga la versión más reciente desde [Releases](https://github.com/yisas97/guit/releases):

| Plataforma | Archivo |
|------------|---------|
| Windows x64 | `guit-windows-x64.exe` |
| Linux x64 | `guit-linux-x64` |
| macOS Intel | `guit-macos-x64` |
| macOS Apple Silicon | `guit-macos-arm64` |

**Linux/macOS:** Después de descargar, dale permisos de ejecución:
```bash
chmod +x guit-linux-x64
./guit-linux-x64
```

### Desde el código fuente

```bash
# Clonar el repositorio
git clone https://github.com/yisas97/guit.git
cd guit

# Compilar e instalar
cargo install --path .
```

### Requisitos

- Rust 1.70+ (edición 2021)
- Git instalado en el sistema

## Uso

Ejecuta `guit` dentro de cualquier repositorio Git:

```bash
cd tu-proyecto-git
guit
```

## Atajos de teclado

### Modo Normal

| Tecla | Acción |
|-------|--------|
| `j` / `↓` | Bajar en la lista de archivos |
| `k` / `↑` | Subir en la lista de archivos |
| `h` / `l` / `Tab` | Alternar entre paneles (archivos/diff) |
| `Space` / `Enter` | Stage/unstage archivo seleccionado |
| `a` | Stage todos los archivos |
| `u` | Unstage todos los archivos |
| `d` | Descartar cambios del archivo seleccionado |
| `c` | Entrar al modo commit |
| `g` | Ver historial de commits (git log) |
| `b` | Ver y cambiar ramas |
| `r` | Refrescar estado |
| `PgUp` / `PgDn` | Scroll rápido en el diff |
| `q` / `Esc` | Salir |

### Modo Commit

| Tecla | Acción |
|-------|--------|
| `Escribir` | Ingresar mensaje de commit |
| `Tab` | Generar mensaje con AI |
| `Enter` | Confirmar commit |
| `Esc` | Cancelar y volver |
| `Backspace` | Borrar carácter |
| `←` / `→` | Mover cursor |

### Modo Log

| Tecla | Acción |
|-------|--------|
| `j` / `↓` | Siguiente commit |
| `k` / `↑` | Commit anterior |
| `q` / `Esc` | Volver al modo normal |

### Modo Branches

| Tecla | Acción |
|-------|--------|
| `j` / `↓` | Siguiente rama |
| `k` / `↑` | Rama anterior |
| `Enter` / `Space` | Cambiar a rama seleccionada |
| `n` | Crear nueva rama |
| `q` / `Esc` | Volver al modo normal |

### Modo Crear Rama

| Tecla | Acción |
|-------|--------|
| `Escribir` | Nombre de la nueva rama |
| `Enter` | Crear rama |
| `Esc` | Cancelar |
| `Backspace` | Borrar carácter |
| `←` / `→` | Mover cursor |

## Estructura del proyecto

```
guit/
├── src/
│   ├── main.rs      # Punto de entrada y event loop
│   ├── app.rs       # Lógica de estado de la aplicación
│   ├── git.rs       # Interfaz con comandos Git
│   └── ui.rs        # Renderizado de la interfaz TUI
├── Cargo.toml       # Configuración del proyecto
└── Cargo.lock       # Dependencias bloqueadas
```

## Dependencias

- [ratatui](https://github.com/ratatui-org/ratatui) - Framework TUI para Rust
- [crossterm](https://github.com/crossterm-rs/crossterm) - Manipulación de terminal multiplataforma

## Desarrollo

```bash
# Ejecutar en modo desarrollo
cargo run

# Compilar versión optimizada
cargo build --release

# Ejecutar tests
cargo test
```

## Roadmap

- [ ] Integración completa con AI para generación de mensajes de commit
- [ ] Soporte para resolución de conflictos de merge
- [ ] Stash interactivo
- [ ] Búsqueda y filtrado de archivos
- [ ] Temas personalizables
- [ ] Soporte para submodules

## Contribuir

Las contribuciones son bienvenidas. Por favor:

1. Haz fork del repositorio
2. Crea una rama para tu feature (`git checkout -b feature/nueva-funcionalidad`)
3. Commitea tus cambios (`git commit -am 'Agrega nueva funcionalidad'`)
4. Push a la rama (`git push origin feature/nueva-funcionalidad`)
5. Abre un Pull Request

## Licencia

Este proyecto está bajo la Licencia MIT. Ver el archivo `LICENSE` para más detalles.

## Autor

**Jesus Campos**

- GitHub: [@yisas97](https://github.com/yisas97)

---

Construido con Rust
