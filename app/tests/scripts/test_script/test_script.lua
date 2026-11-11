-- Handlers receive one table: `opts.packet` and `opts.threshold`.
local required_file = require("required_file")
local some_file = require("folder.some_file")

-- Register by packet constructor, not by a numeric opcode. The constructor
-- carries its own `opcode`, `side` and `state`, so the engine can read them
-- back and reject a registration that names the wrong phase.
engine:register({
    state = "Handshake",
    side = "C2S",
    packet = c2s.handshake.Handshaking,
    own_type = "read-inject",
    is_blocking = true,
}, function(opts)
    required_file:helper()
    some_file:print()
    print("Got packet and options: ", opts)
    print("Registered packet:", c2s.handshake.Handshaking.name, c2s.handshake.Handshaking.opcode)
    print(s2c.play.SetEntityMetadata)

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
    print("Valid particle opcode: ", ok.opcode)

    local inject = update.injection(true)
    inject:push_packet(ok, opts.threshold)
    print("Valid injection len: ", inject.len)

    return update.inject(inject)
end)
