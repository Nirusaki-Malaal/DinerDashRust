property sg, name, events, happyMS, madMS, orderPoints, timeMultiplier, stateTimeMultiplier, minMS, previousState, availableSnacks

on new me, t, dat
  sg = t
  name = #snack
  happyMS = dat.happyMS
  timeMultiplier = 1.0
  stateTimeMultiplier = 1.0
  madMS = dat.madMS
  events = [#putDrinks, #getTicket]
  minMS = dat.minMS
  return me
end

on setSnacks me, ls
  if voidp(ls) then
    ls = []
  end if
  availableSnacks = ls
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
  previousState = s.name
  if availableSnacks.count = 0 then
    sg.setState(previousState)
    return 
  end if
  sg.setWantedSnack(availableSnacks[random(availableSnacks.count)])
  sg.startMoodTimer(max(minMS, happyMS * timeMultiplier * stateTimeMultiplier), max(minMS, madMS * timeMultiplier * stateTimeMultiplier))
end

on event me, sym, arg
  if voidp(sg) then
    return 
  end if
  if sym = #mood then
    if arg = 0 then
      sg.leaveWithoutPaying()
    else
      sg.stopMoodTimer()
      sg.setState(previousState)
    end if
  else
    if sym = #flo then
      if arg.dropOff(sg.wantedSnack, sg.table) then
        sg.stopMoodTimer(1)
        sg.setState(previousState)
        return #Ok
      end if
      if arg.hasPayload(#drinks) then
        arg.dropOff(#drinks, sg.table)
        return #Ok
      end if
      return #nothing_to_do
    end if
  end if
end

on leaveState me, s
  if voidp(sg) then
    return 
  end if
  sg.setWantedSnack(VOID)
  sg.stopMoodTimer()
end
