property sg, name, happyMS, madMS, timeMultiplier, stateTimeMultiplier, minMS, game_mgr

on new me, t, dat
  sg = t
  game_mgr = sg.game_mgr
  happyMS = dat.happyMS
  madMS = dat.madMS
  timeMultiplier = 1.0
  stateTimeMultiplier = 1.0
  moodTimer = sg.mood_timer
  name = #waiting
  minMS = dat.minMS
  return me
end

on setTimeMultiplier me, num
  timeMultiplier = num
end

on setStateTimeMultiplier me, num
  stateTimeMultiplier = num
end

on destroy me
  sg = VOID
end

on enterState me, previousState
  if voidp(sg) then
    return 
  end if
  if not game_mgr.tutorialMode then
    sg.startMoodTimer(max(minMS, happyMS * timeMultiplier * stateTimeMultiplier), max(minMS, madMS * timeMultiplier * stateTimeMultiplier))
  end if
end

on event me, sym, arg
  if voidp(sg) then
    return 
  end if
  if sym = #mood then
    if arg = 0 then
      sg.stopMoodTimer()
      sg.stopWaiting()
    end if
  else
    if sym = #seat then
      sg.stopMoodTimer(1)
      sg.setState(#deciding)
    end if
  end if
end

on leaveState me, nextState
  if voidp(sg) then
    return 
  end if
  sg.stopMoodTimer()
end
