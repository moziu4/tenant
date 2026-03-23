set -e
suffix='' # enter version suffix

if [ -z "$suffix" ]
then
  version=$(date +'%Y%m%d')
  arg=$(date +'%Y.%-m.%-d')
else
  version=$(date +'%Y%m%d')-$suffix
  arg=$(date +'%Y.%-m.%-d')-$suffix
fi

tag=backend/tenant:v$version

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
  --build-arg IMAGE_VERSION="$arg" \
  --secret id=CARGO_CONFIG,src="$HOME"/.cargo/config.toml \
  --secret id=CARGO_CREDEN,src="$HOME"/.cargo/credentials.toml \
  -t "$tag" \
  .

rm .version
printf '\n\n> Built image:  %s\n\n' "$tag"

select action in push rebuild exit; do
  case $action in

  "rebuild")
  bash ./docker.sh
  break
  ;;

  "exit")
  break
  ;;
 esac
done