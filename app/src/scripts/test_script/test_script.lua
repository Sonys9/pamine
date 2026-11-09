-- Handlers receive one table: `opts.packet` and `opts.threshold`.
local helper = require("required_file")

engine:register(0, "read-inject", function(opts)
    helper:helper()
    print("Got packet and options: ", opts)
    
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
