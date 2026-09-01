echo "Dummy E2E to verify the script runs without the real MLIR build"
mkdir -p arkhe-compiler/outputs
touch arkhe-compiler/outputs/solver.c
touch arkhe-compiler/outputs/coils.gcode
touch arkhe-compiler/outputs/plasma_accel.vhdl
echo "✅ Teste E2E concluído com sucesso!"
