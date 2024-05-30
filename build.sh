 curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
 curl -fsSL https://bun.sh/install | bash
 echo "export PATH=$HOME/.cargo/bin:$PATH" >> ~/.bashrc
 source .bashrc
 cargo install just
 touch bundlrs.db
