property happyMS, madMS, minMS, ms, callback, stopped, totalms

on new me, Args
  minMS = Args.minMS
  callback = Args.callback
  return me
end

on destroy me
  unsubscribe(me, #exitFrame)
  stopped = 1
  if not voidp(callback) then
    callback.deleteAll()
  end if
end

on setMadMS me, newms
  ms = max(minMS, ms - (madMS - newms))
  madMS = newms
end

on setHappyMS me, newms
  happyMS = newms
end

on exitFrame me, msdelta
  if stopped then
    return 
  end if
  ms = ms + msdelta
  totalms = totalms + msdelta
  if ms >= madMS then
    if not voidp(callback.target) then
      call(callback.handler, callback.target, callback[#Args])
    end if
    ms = 0
  end if
end

on reset me
  ms = 0
end

on start me
  ms = 0
  stopped = 0
  totalms = 0
  subscribe(me, #exitFrame)
end

on stop me
  stopped = 1
  unsubscribe(me, #exitFrame)
  happy = totalms < happyMS
  ms = 0
  totalms = 0
  return happy
end
