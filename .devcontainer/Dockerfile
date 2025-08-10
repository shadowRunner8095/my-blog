

FROM ubuntu:latest



ENV SHELL=/bin/zsh

RUN apt-get update && apt-get install -y \
    zip \
    unzip \
    git \
    curl \
    zsh \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

ENV PORT=3000

USER ubuntu

WORKDIR /home/ubuntu

# Install Oh My Zsh for zsh (non-interactive)
RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" --unattended
RUN curl -fsSL https://get.pnpm.io/install.sh | sh 
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN curl -fsSL https://fnm.vercel.app/install | zsh

CMD ["tail", "-f", "/dev/null"]