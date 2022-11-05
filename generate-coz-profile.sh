#! /bin/bash
end=$((SECONDS+120))

while [ $SECONDS -lt $end ]; do
    echo $SECONDS
    coz run --- ./target/release-with-debug/kfc-ml > /dev/null 2>&1
done
