#!/bin/bash
for i in {1..1000}
do
   num=$((16640610 + $i))
   cargo run get-host-sub-mat $num 2>/dev/null
   exit_status=$?
   if [[ $exit_status -ne 0 ]]
   then
      echo "!!! FAIL !!!"
   fi
done
