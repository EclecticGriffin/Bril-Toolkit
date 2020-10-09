TURNT_TESTS=("global_dce" "local_dce" "lvn" "orphan" "df")
BRENCH_TESTS=("lvn_bench")
BRENCH_CONFIG_NAME="brench.toml"

for t_test in ${TURNT_TESTS[*]}; do

    cd "$t_test"/
    echo "running tests in $t_test"
    turnt *.bril
    cd - > /dev/null
    echo
done

for b_test in ${BRENCH_TESTS[*]}; do

    cd "$b_test"/
    echo "running tests in $b_test"
    brench "$BRENCH_CONFIG_NAME" | grep 'missing\|incorrect\|timeout'
    cd - > /dev/null
    echo
done

cd $BRENCH_TESTS;
echo "Checking SSA";
for f in *.bril; do
    result=$((bril2json < "$f") | (cargo run -- transform -o to_ssa) 2> /dev/null | is_ssa)

    echo "$f - $result"
done
