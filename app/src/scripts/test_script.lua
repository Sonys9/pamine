-- Handlers receive one table: `opts.packet` and `opts.threshold`.
-- `update` is a plain table of constructors, so it is called with a dot.
-- `Injection` is a userdata, so its methods use a colon.

engine:register(0, "read-inject", function(opts)
    print("Got packet: " .. tostring(opts.packet.value.value.type))
    print("Threshold: " .. tostring(opts.threshold.threshold))

    -- Status packet, the simplest case.
    local response = s2c.status.StatusResponse { json_response = '{"ok":true}' }
    print("StatusResponse opcode: " .. tostring(response.opcode))

    local inject = update.injection(true)
    inject:push_packet(response, opts.threshold)
    print("Injection len: " .. tostring(inject.len))

    return update.inject(inject)
end)

engine:register(1, "read-write", function(opts)
    print("Got packet: " .. tostring(opts.packet.value.value.type))

    -- A play packet with a nested value enum (BossBarAction).
    local bar = s2c.play.BossBar {
        uuid = string.rep("\0", 16),
        action = {
            type = "Add",
            title = "hello",
            health = 1.0,
            color = 0,
            division = 0,
            flags = 0,
        },
    }
    print("BossBar opcode: " .. tostring(bar.opcode))

    local change_with = update.packet(bar, opts.threshold, false)
    print("Injection len from BossBar: " .. tostring(change_with.len))

    return update.change(change_with)
end)
