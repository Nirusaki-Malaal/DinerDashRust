property sg, name, events, happyMS, madMS, timeMultiplier, stateTimeMultiplier, minMS, game_mgr

on new me, t, dat
  sg = t
  game_mgr = sg.game_mgr
  happyMS = dat.happyMS
  madMS = dat.madMS
  minMS = dat.minMS
  timeMultiplier = 1.0
  stateTimeMultiplier = 1.0
  name = #ordered
  events = [#putDrinks, #putFood]
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

on enterState me
  if not game_mgr.tutorialMode then
    sg.startMoodTimer(max(minMS, happyMS * timeMultiplier * stateTimeMultiplier), max(minMS, madMS * timeMultiplier * stateTimeMultiplier))
  end if
  sg.table.showNumber()
end

on event me, sym, flo
  if sym = #mood then
    if flo = 0 then
      sg.leaveWithoutPaying()
    end if
  else
    if sym = #flo then
      if flo.dropOff(#food, sg.table) then
        sg.stopMoodTimer(1)
        sg.setState(#eating)
        return #Ok
      end if
      if flo.hasPayload(#drinks) then
        flo.dropOff(#drinks, sg.table)
        return #Ok
      end if
      return #nothing_to_do
    else
      if sym = #foodPickedUp then
        call(#setFoodPickedUp, sg.sitters, flo)
      end if
    end if
  end if
end

on leaveState me
  if voidp(sg) then
    return 
  end if
  sg.table.hideNumber()
  sg.stopMoodTimer()
end
