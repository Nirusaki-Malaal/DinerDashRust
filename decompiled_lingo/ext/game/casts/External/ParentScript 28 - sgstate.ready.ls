property sg, name, events, happyMS, madMS, orderPoints, timeMultiplier, stateTimeMultiplier, minMS, game_mgr

on new me, t, dat
  sg = t
  game_mgr = sg.game_mgr
  name = #ready
  happyMS = dat.happyMS
  timeMultiplier = 1.0
  stateTimeMultiplier = 1.0
  madMS = dat.madMS
  events = [#putDrinks, #getTicket]
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

on enterState me, s
  if not game_mgr.tutorialMode then
    sg.startMoodTimer(max(minMS, happyMS * timeMultiplier * stateTimeMultiplier), max(minMS, madMS * timeMultiplier * stateTimeMultiplier))
  end if
  if not voidp(sg.table) then
    if not voidp(sg.table) then
      sg.table.closeMenus()
    end if
  end if
end

on event me, sym, arg
  if voidp(sg) then
    return 
  end if
  if sym = #mood then
    if arg = 0 then
      sg.leaveWithoutPaying()
    end if
  else
    if sym = #flo then
      if arg.pickUp(#ticket, sg.table) then
        sg.stopMoodTimer(1)
        sg.setState(#ordered)
        return #Ok
      end if
      if arg.hasPayload(#drinks) then
        arg.dropOff(#drinks, sg.table)
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
  sg.table.hideMenus()
  sg.stopMoodTimer()
end
