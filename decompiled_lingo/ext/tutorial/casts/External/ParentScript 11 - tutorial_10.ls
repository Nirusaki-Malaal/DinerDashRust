property done, sitterGroup
global glob

on startStep me
  done = 0
  t = glob.managers.game.room.tables
  if t.count <> 2 then
    return 
  end if
  sitterGroup = t[2].sitterGroup
  if voidp(sitterGroup) then
    return 
  end if
  return 1
end

on checkDone me
  if done or voidp(sitterGroup) then
    return 
  end if
  done = sitterGroup.state.name <> #check
  return done
end
