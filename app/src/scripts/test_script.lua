print("This is a test script, jit version: " .. jit.version)

for i = 0, 100, 10 do
    print("Imitating a load: " .. i .. "%")
    sleep(100) -- Sleeping 100 ms
end

print("Loaded")