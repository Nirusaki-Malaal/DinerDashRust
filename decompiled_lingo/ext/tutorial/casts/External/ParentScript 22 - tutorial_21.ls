property done, ms, sitterGroups
global glob

on startStep me
  done = 0
  sitterGroups = glob.managers.game.room.sitterGroups
  if sitterGroups.count <> 2 then
    return 
  end if
  ms = the milliSeconds + 2000
  return 1
end

on checkDone me
  if done then
    return 
  end if
  doneEating = 1
  repeat with i in sitterGroups
    doneEating = doneEating and (i.state.name = #check)
  end repeat
  done = (the milliSeconds > ms) and doneEating
  return done
end
