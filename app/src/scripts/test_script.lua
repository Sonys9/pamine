-- Handlers receive one table: `opts.packet` and `opts.threshold`.

engine:register(0, "read-inject", function(opts)
    print("Got packet: " .. tostring(opts.packet.value.value.type))

    -- Correct pair: particle_id 24 (dust) with Dust payload.
    local ok = s2c.play.SetEntityMetadata {
        entity_id = 1,
        metadata = {
            {
                index = 0,
                value = {
                    type = "Particle",
                    particle_id = 24,
                    data = { type = "Dust", red = 1.0, green = 0.0, blue = 0.0, scale = 1.0 },
                },
            },
        },
    }
    print("Valid particle opcode: " .. tostring(ok.opcode))

    local inject = update.injection(true)
    inject:push_packet(ok, opts.threshold)
    print("Valid injection len: " .. tostring(inject.len))

    return update.inject(inject)
end)

engine:register(1, "read-write", function(opts)
    print("Got packet: " .. tostring(opts.packet.value.value.type))

    -- Mismatched pair: particle_id 24 (dust) but BlockState payload.
    -- This must be rejected, not written.
    local bad = s2c.play.SetEntityMetadata {
        entity_id = 1,
        metadata = {
            {
                index = 0,
                value = {
                    type = "Particle",
                    particle_id = 24,
                    data = { type = "BlockState", value = 5 },
                },
            },
        },
    }
    print("Mismatched particle opcode: " .. tostring(bad.opcode))

    local change_with = update.packet(bad, opts.threshold, false)
    print("Mismatched injection len: " .. tostring(change_with.len))

    return update.change(change_with)
end)
