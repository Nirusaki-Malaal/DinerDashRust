property glob, old_ms, theframe, evt_mgr, initialized, paused, pauseTime, lastPause, showframerate, frameCount, totalms
global numcalls, totaltime, avgtime, maxtime

on new me, g
  glob = g
  showframerate = 0
  return me
end

on init me
  evt_mgr = glob.managers.event
  theframe = marker("play")
  initialized = 1
end

on destroy me
  evt_mgr = VOID
  glob = VOID
end

on getMS me
  if paused then
    return lastPause
  end if
  return the milliSeconds - pauseTime
end

on goFrame me, fr
  if stringp(fr) then
    fr = marker(fr)
  end if
  theframe = fr
end

on showframerate me, b
  totalms = 0
  frameCount = 0
  showframerate = b
end

on setPaused me, bool
  if paused = bool then
    return 
  end if
  paused = bool
  if bool then
    lastPause = the milliSeconds
  else
    pauseTime = pauseTime + (the milliSeconds - lastPause)
  end if
end

on exitFrame me
  if not voidp(theframe) then
    go(theframe)
  end if
  if paused then
    return 
  end if
  if not initialized then
    return 
  end if
  ms = me.getMS()
  delta = ms - old_ms
  evt_mgr.sendEvent(#exitFrame, delta)
  old_ms = ms
  if showframerate then
    frameCount = frameCount + 1
    totalms = totalms + delta
    if frameCount = 100 then
      put 100.0 / (totalms * 0.001) && "fps"
      totalms = 0
      frameCount = 0
    end if
  end if
  return 
end
