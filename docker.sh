set -e

# ── Configuración reutilizable ───────────────────────────────────────────────
DOCKER_USER="moziu4"   # nombre de usuario en Docker Hub
SERVICE_NAME="tenant"  # nombre del servicio / repositorio
# ─────────────────────────────────────────────────────────────────────────────

# TODO: soporte de versiones con fecha está preparado pero desactivado por ahora
# Cuando se active, añadir suffix y arg aquí para etiquetar con fecha/versión.
tag="$DOCKER_USER/$SERVICE_NAME:latest"

{
  printf "Tag:  %s\n" "$tag"
  printf "Date: %s\n" "$(date --rfc-3339=seconds)"
  printf "Machine: %s\n" "$(uname -n)"
  printf "Architecture: %s\n" "$(uname -om)"
  printf "User: %s\n" "$(whoami)"
  printf "\n\nCommits:\n"
  git log --pretty=format:'%h %cI %<(15)%an %s' -20

  printf "\n\nDocker:\n"
  docker version

  printf "\n\nCargo tree:\n"
  cargo tree -e no-dev,features -f "{p} f={f}"
} > .version

time DOCKER_BUILDKIT=1 docker build --pull \
  --ssh default="$HOME"/.ssh/id_ed25519 \
  --secret id=CARGO_CONFIG,src="$HOME"/.cargo/config.toml \
  --secret id=CARGO_CREDEN,src="$HOME"/.cargo/credentials.toml \
  -t "$tag" \
  .

rm .version
printf '\n\n> Built image:  %s\n\n' "$tag"

select action in push rebuild exit; do
  case $action in

  "push")
  docker push "$tag"
  break
  ;;

  "rebuild")
  bash ./docker.sh
  break
  ;;

  "exit")
  break
  ;;
  esac
done