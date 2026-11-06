-- Handlers receive one table: `opts.packet` and `opts.threshold`.
-- `update` is a plain table of constructors, so it is called with a dot:
-- `update.inject(x)`, never `update:inject(x)`.
-- `Injection` is a userdata, so its methods use a colon: `inj:push_frame(s)`.

engine:register(0, "read-inject", function(opts)
    print("Got packet: " .. tostring(opts.packet.type))
    print("Threshold: " .. tostring(opts.threshold.threshold))

    local inject = update.injection(true)
    inject:push_frame("hello")
    print("Injection len: " .. tostring(inject.len) .. ", changeable: " .. tostring(inject.changeable))

    return update.inject(inject)
end)

engine:register(1, "read-write", function(opts)
    print("Got packet: " .. tostring(opts.packet.type))
    print("Threshold: " .. tostring(opts.threshold.threshold))

    local change_with = update.injection(false)
    change_with:push_frame("replaced")

    return update.change(change_with)
end)
