./configure \
    --libdir="." \
    --tccdir="." \
    --includedir="./include" &&

make

$tccfolder="target/debug"

cp tcc $tccfolder/tcc
cp libtcc1.a $tccfolder/libtcc1.a
mkdir $tccfolder/include