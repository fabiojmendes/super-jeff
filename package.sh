echo "Running packaging script"
FILE_NAME="super-jeff-$TRAVIS_TAG-$TRAVIS_OS_NAME"

echo "Packaging $FILE_NAME"

if [[ "$TRAVIS_OS_NAME" == "windows" ]]; then
  ARTIFACT="$FILE_NAME.zip"
  mkdir super-jeff
  cp target/release/super-jeff.exe ./super-jeff
  cp -r assets ./super-jeff
  cp lib/*.{dll,txt} ./super-jeff
  ls -l super-jeff
  tar -cavf $ARTIFACT super-jeff
else
  tar -C target/release -cvf $FILE_NAME.tar super-jeff
  tar -rvf $FILE_NAME.tar assets
  gzip -f $FILE_NAME.tar
  ARTIFACT="$FILE_NAME.tar.gz"
fi

echo "$ARTIFACT generated"
