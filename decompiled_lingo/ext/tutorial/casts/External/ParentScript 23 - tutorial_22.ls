property done, sitterGroups
global glob

on startStep me
  done = 0
  sitterGroups = glob.managers.game.room.sitterGroups
  if sitterGroups.count <> 2 then
    return 
  end if
  if (sitterGroups[1].state.name <> #check) or (sitterGroups[2].state.name <> #check) then
    return 
  end if
  return 1
end

on checkDone me
  if done then
    return 
  end if
  ch = 0
  repeat with s in sitterGroups
    ch = ch or (s.state.name = #check)
  end repeat
  done = not ch
  return done
end
