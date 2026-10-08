on timedCall ms, cb, grp
  t = script("timeout").new(ms, cb, 1, grp)
  return t
end

on triggerCall t, cb
  n = script("notifier", "scripts.trigger").new()
  n.setCallback(cb)
  n.addTrigger(t)
  n.start()
  return n
end
