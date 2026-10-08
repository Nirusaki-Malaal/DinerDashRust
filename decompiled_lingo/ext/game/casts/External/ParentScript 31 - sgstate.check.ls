property sg, name, happyMS, madMS, timeMultiplier, stateTimeMultiplier, minMS, game_mgr

on new me, t, dat
  sg = t
  game_mgr = sg.game_mgr
  happyMS = dat.happyMS
  madMS = dat.madMS
  timeMultiplier = 1.0
  stateTimeMultiplier = 1.0
  name = #check
  minMS = dat.minMS
  return me
end

on destroy me
  sg = VOID
end

on setTimeMultiplier me, num
  timeMultiplier = num
end

on setStateTimeMultiplier me, num
  stateTimeMultiplier = num
end

on enterState me, s
  if not game_mgr.tutorialMode then
    sg.startMoodTimer(max(minMS, happyMS * timeMultiplier * stateTimeMultiplier), max(minMS, madMS * timeMultiplier * stateTimeMultiplier))
  end if
  call(#setReady, sg.sitters, 1)
end

on event me, sym, flo
  if sym = #mood then
    if flo = 0 then
      sg.leaveWithoutPaying()
    end if
  else
    if sym = #flo then
      if flo.canPickUp() then
        sg.stopMoodTimer(1)
        sg.table.showCheck()
        sg.setState(#scoring)
        flo.dropCheck(sg.table)
        return #Ok
      end if
      return #hands_full
    end if
  end if
end

on leaveState me, s
  if voidp(sg) then
    return 
  end if
  call(#setReady, sg.sitters, 0)
  sg.stopMoodTimer()
end
