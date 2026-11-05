print("Running lua code")

engine:register(0, "read", function(packet_data)
    print("Got packet: " .. packet_data)
end)