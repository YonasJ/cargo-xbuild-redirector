
set -e

CARGO=`rustup which cargo`
CARGO_BAK="${CARGO}.bak"
if [ -f $CARGO_BAK ]; then
  echo backup exists at $CARGO_BAK. Updating cargo to backup.
  cp $CARGO_BAK $CARGO
else
  echo no back exists at $CARGO_BAK. Making a backup.
  cp $CARGO $CARGO_BAK
fi

cargo build --release

cargo run install

echo install complete.

