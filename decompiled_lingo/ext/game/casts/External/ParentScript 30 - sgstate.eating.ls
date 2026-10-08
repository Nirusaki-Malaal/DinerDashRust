property sg, name, timer, events, timeMultiplier, stateTimeMultiplier, durationMS, minMS, availableSnacks, snackPercent, snackMS

on new me, g, dat
  sg = g
  durationMS = dat.durationMS
  name = #eating
  timeMultiplier = 1.0
  stateTimeMultiplier = 1.0
  events = [#putDrinks]
  availableSnacks = []
  minMS = dat.minMS
  return me
end

on destroy me
  me.destroyTimer()
  sg = VOID
end

on setSnacks me, ls, pc
  if voidp(ls) then
    ls = []
  end if
  availableSnacks = ls
  snackPercent = integer(pc)
end

on setTimeMultiplier me, num
  timeMultiplier = num
end

on setStateTimeMultiplier me, num
  stateTimeMultiplier = num
end

on destroyTimer me
  if not voidp(timer) then
    timer.destroy()
    timer = VOID
  end if
end

on enterState me, previousState
  t = sg.table
  if voidp(t) then
    return 
  end if
  me.destroyTimer()
  if previousState.name = #snack then
    t.setFoodPaused(0)
    timer = timedCall(max(minMS, (durationMS * timeMultiplier * stateTimeMultiplier) - snackMS), [#target: sg, #handler: #setState, #Args: #check])
  else
    if (random(100) < snackPercent) and availableSnacks.count and ((durationMS * timeMultiplier * stateTimeMultiplier) >= 2000) then
      jitter = durationMS - 2000
      snackMS = durationMS - jitter + random(jitter)
      timer = timedCall(snackMS, [#target: sg, #handler: #setState, #Args: #snack])
      t.placeFood(durationMS)
    else
      timer = timedCall(max(minMS, durationMS * timeMultiplier * stateTimeMultiplier), [#target: sg, #handler: #setState, #Args: #check])
      t.placeFood(durationMS)
    end if
  end if
end

on event me, sym, arg
  if sym = #mood then
    if sg.mood = 0 then
      timer.stop()
      sg.leaveWithoutPaying()
    end if
  else
    if sym = #flo then
      if arg.hasPayload(#drinks) then
        arg.dropOff(#drinks, sg.table)
        return #Ok
      end if
      return #still_eating
    end if
  end if
end

on leaveState me, nextState
  if voidp(sg) then
    return 
  end if
  if nextState.name <> #snack then
    sg.table.hideFood()
  else
    sg.table.setFoodPaused(1)
  end if
  me.destroyTimer()
end
