# Contribuir a guit

Gracias por tu interés en contribuir a guit.

## Desarrollo local

```bash
# Clonar el repositorio
git clone https://github.com/yisas97/guit.git
cd guit

# Ejecutar en modo desarrollo
cargo run

# Compilar versión optimizada
cargo build --release

# Ejecutar tests
cargo test
```

## Flujo de trabajo

1. Haz fork del repositorio
2. Crea una rama para tu feature (`git checkout -b feature/nueva-funcionalidad`)
3. Commitea tus cambios (`git commit -am 'Agrega nueva funcionalidad'`)
4. Push a la rama (`git push origin feature/nueva-funcionalidad`)
5. Abre un Pull Request

## Crear una nueva release (solo mantenedores)

Las releases se generan automáticamente con GitHub Actions al crear un tag:

```bash
# Asegúrate de estar en la rama principal con todo commiteado
git checkout main
git pull origin main

# Crear tag con la versión
git tag v0.1.0

# Push del tag (esto dispara el workflow)
git push origin v0.1.0
```

El workflow automáticamente:
- Compila para Windows, Linux y macOS (Intel + Apple Silicon)
- Crea una GitHub Release con todos los binarios
- Genera notas de release basadas en los commits

## Estructura del proyecto

```
guit/
├── src/
│   ├── main.rs      # Punto de entrada y event loop
│   ├── app.rs       # Lógica de estado de la aplicación
│   ├── git.rs       # Interfaz con comandos Git
│   └── ui.rs        # Renderizado de la interfaz TUI
├── .github/
│   └── workflows/
│       └── release.yml  # CI/CD para releases
├── Cargo.toml       # Configuración del proyecto
└── Cargo.lock       # Dependencias bloqueadas
```

## Estilo de código

- Sigue las convenciones de Rust (rustfmt)
- Usa `cargo clippy` para verificar el código
- Escribe mensajes de commit descriptivos
