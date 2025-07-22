mkdir -p txt_chunks
for f in chunk*.hex; do
    out="txt_chunks/${f%.hex}.txt"
    # convert hex to binary
    xxd -r -p "$f" > "$out"
done
echo "Converted all chunk*.hex to binary files in bin_chunks/"



