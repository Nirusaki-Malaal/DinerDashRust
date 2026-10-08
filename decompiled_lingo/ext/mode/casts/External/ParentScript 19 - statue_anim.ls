property statue1, statue2, door, mgr, fadeSpeed, doorDelay, doorSpeed, doorStarted, statue2Started, doorDone, msCount, currentAlpha, alphadir, doorhome, doorstart, doorY, statuemems, currentMem, statueCover, done

on new me, Args
  mgr = Args.mgr
  statue1 = Args.statue1
  statue2 = Args.statue2
  door = Args.door
  doorhome = door.getLoc()[2]
  doorstart = doorhome - 500
  statuemems = []
  repeat with i = 1 to 7
    statuemems.add(member("statue_" & i, "gfx.screens"))
  end repeat
  statueCover = Args.statueCover
  doorSpeed = 200
  doorDelay = 1000
  fadeSpeed = 5
  return me
end

on destroy me
  unsubscribe(me, #exitFrame, "titles")
  statues = VOID
  door = VOID
  mgr = VOID
end

on start me
  SndSFX("sfx_goddess_appear")
  subscribe(me, #exitFrame, "titles")
  door.setY(doorstart)
  door.setVisible(1)
  statue1.setVisible(1)
  statue1.setAlpha(0)
  statue2.setVisible(1)
  statue2.setAlpha(0)
  statue1.setMember(statuemems[1])
  statue2.setMember(statuemems[3])
  currentMem = 3
  currentAlpha = 1
  alphadir = 1
  statue2Started = 0
  msCount = 0
  done = 0
end

on stop me
  done = 1
  unsubscribe(me, #exitFrame, "titles")
  door.setZOffset(0)
  mgr.statueDone()
end

on exitFrame me, ms
  if done then
    return 
  end if
  if doorDone then
    currentAlpha = min(100, currentAlpha + fadeSpeed)
    statueCover.setAlpha(currentAlpha)
    statue1.setAlpha(100 - currentAlpha)
    statue2.setAlpha(100 - currentAlpha)
    if currentAlpha = 100 then
      me.stop()
    end if
    return 
  end if
  statue1.setAlpha(currentAlpha)
  if statue2Started then
    statue2.setAlpha(100 - currentAlpha)
  end if
  currentAlpha = max(0, min(currentAlpha + (alphadir * fadeSpeed), 100))
  if (currentAlpha = 0) or (currentAlpha = 100) then
    alphadir = -alphadir
    if not statue2Started then
      statue2Started = 1
    end if
    if currentAlpha = 0 then
      if not doorStarted then
        if currentMem < 7 then
          currentMem = currentMem + 1
        else
          doorStarted = 1
          msCount = 0
        end if
      end if
    end if
    statue1.setMember(statuemems[currentMem])
  else
    statue2.setMember(statuemems[min(statuemems.count, currentMem + 1)])
  end if
  if not doorStarted then
    return 
  end if
  if doorDone then
    return 
  end if
  msCount = msCount + ms
  doorY = min(doorhome, doorstart + integer(doorSpeed * msCount * 0.001))
  if doorY = doorhome then
    doorDone = 1
    currentAlpha = 0
    statueCover.setAlpha(0)
    statueCover.setVisible(1)
  end if
  door.setY(doorY)
end
