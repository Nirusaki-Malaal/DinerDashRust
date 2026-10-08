property done, kitchen, sitterGroup, flo
global glob

on startStep me
  done = 0
  rm = glob.managers.game.room
  flo = rm.waiter
  t = rm.tables
  kitchen = rm.kitchen
  if t.count <> 2 then
    return 
  end if
  sitterGroup = t[2].sitterGroup
  if sitterGroup.voidp then
    return 
  end if
  if me.checkDone() then
    return 
  end if
  return 1
end

on checkDone me
  if done or voidp(sitterGroup) then
    return 
  end if
  done = (sitterGroup.state.name <> #deciding) and kitchen.getMakingOrMade(sitterGroup.table)
  if done then
    nothing()
  end if
  return done
end
