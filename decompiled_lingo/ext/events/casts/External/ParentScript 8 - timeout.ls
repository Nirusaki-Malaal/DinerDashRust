property eventGroup, timeleft, period, callback, paused, oneshot, running, group
global glob

on new me, ms, cb, one, grp
  group = grp
  period = ms
  callback = cb
  timeleft = ms
  oneshot = one
  if not voidp(cb) and not voidp(ms) then
    me.start()
  else
    paused = 1
  end if
  return me
end

on setGroup me, grp
  if running then
    me.stop()
  end if
  group = grp
end

on destroy me
  target = VOID
  callback = VOID
  unsubscribe(me, #exitFrame, group)
  running = 0
end

on fire me
  if voidp(callback) then
    return 
  end if
  call(callback.handler, callback.target, callback[#Args])
  if oneshot then
    me.stop()
  else
    timeleft = period
  end if
end

on getMS me
  return period - timeleft
end

on setPaused me, p
  paused = p
end

on setCallback me, cb
  callback = cb
end

on setPeriod me, ms
  period = ms
end

on start me
  timeleft = period
  paused = 0
  subscribe(me, #exitFrame, group)
  running = 1
end

on stop me
  timeleft = period
  paused = 1
  unsubscribe(me, #exitFrame, group)
  running = 0
end

on exitFrame me, ms
  if paused or not running then
    return 
  end if
  timeleft = timeleft - ms
  if timeleft < 0 then
    if not voidp(callback) then
      me.fire()
    end if
  end if
end
