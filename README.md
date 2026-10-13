# Hello Solana 🦀 — Guía práctica de Anchor, Surfpool y LiteSVM

> Un proyecto de aprendizaje reproducible: desde Windows 11 + WSL2 hasta compilar, desplegar y probar un programa de Solana. Basado en la experiencia de la **Week 2 del Solana Bootcamp de Encode Club**.
>
> **Estado:** compilación ✅ · despliegue local en Surfpool ✅ · test LiteSVM con sBPFv2 ✅ · ejecución de `initialize`/`increment` en Surfpool desde un cliente ⏳

## 1. ¿Qué vamos a construir?

`hello-solana` es un programa de contador creado con **Anchor**. Tiene dos instrucciones:

- `initialize`: crea una cuenta PDA para el contador, inicializa `count = 0`, registra la autoridad y transfiere **1 lamport** desde el pagador a esa cuenta (además de financiar la creación de la cuenta).
- `increment`: solo permite a la autoridad registrada aumentar `count`, hasta un máximo de **10**.

Su objetivo es practicar cuentas, PDA, firmas, validaciones, CPI, compilación sBPF, despliegue local y tests automatizados.

**Importante:** este es un entorno de aprendizaje. Los SOL de Surfpool son simulados; no usaremos billeteras ni fondos de Mainnet.

## 2. Entorno probado

El siguiente conjunto **funcionó para compilar/desplegar localmente**. La prueba LiteSVM necesitó una compilación sBPFv2 (ver sección 8).

| Componente | Versión observada |
| --- | --- |
| Windows | 11 Pro |
| WSL | 3.0.1.0; WSL2 |
| Ubuntu | 24.04.5 LTS |
| Rust / Cargo globales | 1.99.0 |
| Rust del proyecto (`rust-toolchain.toml`) | 1.89.0 |
| Solana CLI activa | 3.1.10 (seleccionada por AVM) |
| Anchor CLI / AVM | 1.2.1 |
| Node.js / npm en Ubuntu | 24.21.0 / 11.19.0 |
| nvm | 0.40.8 |
| Corepack / Yarn | 0.36.0 / 1.22.22 |
| Surfpool | 1.6.1 |
| LiteSVM | 0.10.0 (dependencia de desarrollo) |

**Nota sobre versiones:** inicialmente se instaló Solana CLI 4.3.0, pero al ejecutar `avm use 1.2.1`, AVM seleccionó Solana CLI 3.1.10. Verificar siempre con `solana --version` y `avm solana resolve`. Las versiones futuras podrían requerir ajustes.

## 3. Organización de carpetas

Para evitar problemas de permisos y rendimiento, los proyectos Linux se guardan **dentro de Ubuntu**, no en `/mnt/c` ni `/mnt/d`:

```text
Windows
└── D:\DEV\                  # proyectos habituales de Windows

Ubuntu / WSL2
└── /home/<usuario>/
    ├── .cargo/               # Rust/Cargo y AVM
    ├── .rustup/              # toolchains de Rust
    ├── .nvm/                 # Node.js de Ubuntu
    ├── .local/               # binarios Solana/Surfpool
    └── dev/solana/encode-bootcamp/
        └── hello-solana/     # proyecto Anchor
```

Usá tu **propio nombre de usuario Linux** al repetir esta guía. En nuestra instalación fue `daniela`.

## 4. Preparación de Windows y Ubuntu

### 4.1 WSL2 + Ubuntu

En **PowerShell como administrador**:

```powershell
wsl --install -d Ubuntu-24.04
```

Si Windows lo solicita, reiniciar. Crear usuario y contraseña en Ubuntu. Para comprobar desde PowerShell:

```powershell
wsl --list --verbose
wsl --set-default Ubuntu-24.04
```

Si ya tenés WSL/Ubuntu instalados, **no repitas la instalación**. Comprobá la versión primero.

### 4.2 Paquetes base

En Ubuntu:

```bash
sudo apt update
sudo apt upgrade
sudo apt install -y build-essential pkg-config libssl-dev
```

Verificar:

```bash
gcc --version
make --version
pkg-config --version
```

### 4.3 Crear espacio de trabajo

```bash
mkdir -p ~/dev/solana/encode-bootcamp
cd ~/dev/solana/encode-bootcamp
```

## 5. Instalación de herramientas

> Los scripts remotos cambian con el tiempo. Antes de ejecutarlos, verificá su procedencia y la documentación oficial. Si ya tenés instalada una herramienta, comprobá su versión antes de reinstalarla.

### 5.1 Rust y Cargo

Instalador oficial de Rust: <https://rustup.rs/>.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
rustc --version
cargo --version
```

El proyecto incluye un `rust-toolchain.toml` que solicita Rust **1.89.0**. `rustup` lo instala/activa al trabajar en el proyecto cuando sea necesario. No hace falta cambiar la versión global de Rust.

### 5.2 Solana CLI

Consultá <https://solana.com/docs/intro/installation> y el instalador de Anza. El siguiente comando instala el canal estable, **no una versión fija**:

```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
solana --version
```

**Para reproducir exactamente este proyecto**, usar la versión recomendada/seleccionada por AVM y comprobarla. No actualizar a ciegas.

### 5.3 AVM y Anchor CLI

En nuestra instalación se utilizó esta etiqueta de código fuente:

```bash
cargo install --git https://github.com/otter-sec/anchor --tag v1.2.1 avm --locked
avm install 1.2.1
avm use 1.2.1
anchor --version
avm solana resolve
solana --version
```

AVM configuró **Anchor 1.2.1** y recomendó **Solana CLI 3.1.10**.

### 5.4 Node.js con nvm y Yarn con Corepack

Instalador de nvm utilizado:

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.8/install.sh | bash
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"
nvm install 24
node --version
npm --version
```

Comprobar que ambos ejecutables pertenecen a Ubuntu:

```bash
which node
which npm
```

Activar Yarn Classic con Corepack:

```bash
corepack install -g yarn@1.22.22
corepack enable
yarn --version
```

En WSL es posible que se vean rutas de Node/npm de **Windows** en `PATH`. Verificar que `which node` y `which npm` apunten a `~/.nvm/versions/node/...`.

### 5.5 Surfpool

Documentación: <https://docs.surfpool.run/>.

```bash
curl --proto '=https' --tlsv1.2 -sSfL https://run.surfpool.run/ | bash
source ~/.bashrc
surfpool --version
```

En nuestra instalación Surfpool quedó en `~/.local/bin/surfpool`.

## 6. Crear, compilar y desplegar el proyecto

### 6.1 Inicializar Anchor

Si creás el proyecto desde cero:

```bash
cd ~/dev/solana/encode-bootcamp
anchor init hello-solana
cd hello-solana
```

**Si estás usando una copia o clone de este repositorio, no vuelvas a ejecutar `anchor init`**: entrá en la carpeta existente.

La plantilla de Anchor 1.2.1 creó, entre otros, `Anchor.toml`, `Cargo.toml`, `rust-toolchain.toml`, `programs/`, `app/` y `target/`.

### 6.2 Compilar

```bash
anchor build
ls -lh target/deploy/
```

El binario del programa es `target/deploy/hello_solana.so`. El archivo `hello_solana-keypair.json` en esa misma carpeta contiene una **clave privada del programa**; nunca debe publicarse.

### 6.3 Red local y wallet

**Terminal A (dejar abierta):**

```bash
surfpool start
```

**Terminal B:**

```bash
solana config set --url http://127.0.0.1:8899
solana config get
```

Si todavía **no** existe una wallet exclusivamente de desarrollo:

```bash
solana-keygen new --outfile ~/.config/solana/id.json
```

**No ejecutes este comando si ya tenés una wallet en esa ruta**: primero respaldala y verificá su propósito. Nunca compartas el archivo JSON ni la frase de recuperación.

Verificaciones:

```bash
solana address
solana balance
solana block-height --url http://127.0.0.1:8899
```

Los SOL obtenidos en Surfpool son **simulados** y el estado local puede cambiar al reiniciar la red.

### 6.4 Verificar identidad y desplegar

Antes de desplegar, comprobar:

```bash
solana config get
solana-keygen pubkey target/deploy/hello_solana-keypair.json
anchor keys list
```

Los Program IDs deben coincidir con los declarados en `Anchor.toml` y `declare_id!`. En **esta instancia** fueron:

```text
2J9VuAGzdTeU9SiFhv8hXYV8PgezciDbD6dk31rGJBZk
```

El ID es específico de este proyecto generado; si recreás el proyecto desde cero, probablemente sea diferente.

Desplegar en el localnet definido en `Anchor.toml`:

```bash
anchor program deploy
```

En la experiencia original se utilizó `anchor deploy`, que funcionó, pero imprimió una advertencia de deprecación recomendando `anchor program deploy`.

Verificar:

```bash
solana program show <TU_PROGRAM_ID>
```

En nuestra ejecución, Solana CLI confirmó Program ID, autoridad, cuenta ProgramData y tamaño del binario. **No hemos desplegado este proyecto en Devnet ni en Mainnet.**

## 7. ¿Cómo funciona el contador?

`state.rs` define una cuenta `Counter` con:

```rust
pub struct Counter {
    pub count: u64,
    pub authority: Pubkey,
}
```

- `initialize`: crea una **PDA** derivada con la semilla `b"counter"`, pone `count = 0` y guarda la clave pública del pagador como autoridad.
- `increment`: requiere la firma de la autoridad y que `count < 10`; entonces aumenta el contador en 1.
- `Unauthorized` y `CounterOverflow` son errores definidos por el programa.
- La llamada al System Program desde `initialize` es una **CPI** (invocación entre programas).

**Atención:** desplegar un programa **no ejecuta automáticamente** `initialize` ni `increment`. La prueba automatizada de la sección siguiente usa un simulador **independiente de Surfpool**.

## 8. Testing con LiteSVM: error encontrado y solución de diagnóstico

El test generado en `programs/hello-solana/tests/test_initialize.rs`:

1. Crea un simulador LiteSVM y un pagador.
2. Carga el binario `.so`.
3. Ejecuta `initialize`, comprueba `count == 0` y la autoridad.
4. Ejecuta `increment` y comprueba `count == 1`.

### 8.1 El fallo observado

Con el binario inicialmente compilado en **sBPFv3**, la prueba:

```bash
cargo test --test test_initialize -- --nocapture
```

falló en:

```rust
svm.add_program(program_id, bytes).unwrap();
```

con `Instruction(InvalidAccountData)`. Es decir, el error ocurría **al cargar el programa en LiteSVM**, antes de enviar transacciones.

Inspeccionamos el binario:

```bash
readelf -h target/deploy/hello_solana.so | grep -E 'Class:|Machine:|Flags:'
```

Resultado inicial: `Flags: 0x3` (**sBPFv3**).

### 8.2 Prueba controlada con sBPFv2

Guardamos una copia local del binario original (en `target/`, ignorado por Git):

```bash
cp target/deploy/hello_solana.so target/deploy/hello_solana-sbpfv3-backup.so
```

Compilamos para arquitectura v2:

```bash
anchor build --arch v2
readelf -h target/deploy/hello_solana.so | grep -E 'Class:|Machine:|Flags:'
```

El nuevo archivo mostró `Flags: 0x2`. Repetimos:

```bash
cargo test --test test_initialize -- --nocapture
```

**Resultado observado:** `1 passed; 0 failed`.

**Conclusión:** la comparación proporciona evidencia sólida de una incompatibilidad entre el binario sBPFv3 y **LiteSVM 0.10.0** en este entorno. **No** demuestra que todos los programas sBPFv3 fallen en todas las versiones de LiteSVM.

**Pendiente:** evaluar una versión actualizada de LiteSVM compatible con sBPFv3. Mantener `--arch v2` únicamente como alternativa de diagnóstico, no como decisión universal para proyectos nuevos.

**Importante:** la recompilación para tests no volvió a desplegar el programa. El despliegue que verificamos en Surfpool correspondía al binario anterior.

## 9. Git y seguridad

```bash
git branch -m main
git status
```

Configurá la identidad Git **dentro de Ubuntu** (independiente de Windows):

```bash
git config --global user.name "Tu Nombre"
git config --global user.email "tu-correo@example.com"
```

Antes de hacer commits, revisar `.gitignore`, archivos preparados y claves:

```bash
git add .
git diff --cached --stat
git ls-files --cached | grep -E 'keypair|id\.json|\.env' || true
```

**Nota:** la comprobación por nombres no reemplaza una auditoría de secretos. Revisá archivos nuevos antes de publicarlos; usá una dirección `noreply` de GitHub si preferís no publicar tu correo.

Nuestro primer commit local fue:

```text
5d3f3ee chore: initialize Anchor project
```

El proyecto todavía **no está publicado en GitHub**.

## 10. Problemas conocidos

- **`systemd-binfmt.service` falla en WSL 3.0.1:** observamos `Failed to flush binfmt_misc rules ... Read-only file system`. El entorno logró compilar y desplegar igualmente. No desactivar servicios ni modificar permisos sin investigar el problema concreto.
- **`agave-install info` falla al leer `~/.config/solana/install/config.yml`:** observamos `missing field json_rpc_url`. La CLI y el despliegue funcionaron; el problema del gestor de instalaciones quedó **pendiente de reparación**.
- **`npm` apunta a Windows:** configurar Node con nvm dentro de Ubuntu y verificar `which node` y `which npm`.
- **`solana` cambia de versión tras `avm use`:** ejecutar `avm solana resolve`, `solana --version` y `readlink -f "$(which solana)"` antes de diagnosticar un conflicto.
- **`anchor init` tarda sin mostrar progreso:** esperar y verificar si se creó la carpeta antes de repetir el comando.

## 11. Próximas mejoras

- [ ] Ejecutar instrucciones `initialize` e `increment` desde un cliente y verificar el estado en Surfpool.
- [ ] Agregar tests de autorización (`Unauthorized`) y límite (`CounterOverflow`).
- [ ] Probar una versión reciente de LiteSVM para sBPFv3.
- [ ] Crear scripts reproducibles de validación y documentar la IDE (VS Code + WSL).
- [ ] Añadir capturas y notas de aprendizaje de la Week 2.

## Recursos

- [Solana Developers](https://solana.com/docs)
- [Anchor Documentation](https://www.anchor-lang.com/docs)
- [Surfpool Documentation](https://docs.surfpool.run)
- [Rustup](https://rustup.rs/)
- [nvm](https://github.com/nvm-sh/nvm)

---

**Nota educativa:** este README documenta versiones y resultados observados en una instalación concreta (octubre de 2026). Antes de reutilizar comandos en otra máquina, revisar sus requisitos actuales y adaptar las rutas, versiones y claves al nuevo entorno.
