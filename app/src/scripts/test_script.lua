-- Handlers receive one table: `opts.packet` and `opts.threshold`.
-- `update` is a plain table of constructors, so it is called with a dot.
-- `Injection` is a userdata, so its methods use a colon.

engine:register(0, "read-inject", function(opts)
    print("Got packet: " .. tostring(opts.packet.value.value.type))
    print("Threshold: " .. tostring(opts.threshold.threshold))

    -- Build a real packet from Lua. The opcode comes from the variant's
    -- #[id(N)], so it is always correct.
    local response = s2c.status.StatusResponse { json_response = '{"ok":true}' }
    print("Built packet opcode: " .. tostring(response.opcode))

    local inject = update.injection(true)
    inject:push_packet(response, opts.threshold)
    print("Injection len after push_packet: " .. tostring(inject.len))

    return update.inject(inject)
end)

engine:register(1, "read-write", function(opts)
    print("Got packet: " .. tostring(opts.packet.value.value.type))

    -- The one-call form, equivalent to Injection::from_packet.
    local ping = c2s.status.PingRequest { payload = 12345 }
    local change_with = update.packet(ping, opts.threshold, false)
    print("Injection len from update.packet: " .. tostring(change_with.len))

    return update.change(change_with)
end)
