property done, ms, table
global glob

on startStep me
  done = 0
  t = glob.managers.game.room.tables
  if t.count <> 2 then
    return 
  end if
  table = t[2]
  ms = the milliSeconds + 2000
  return 1
end

on checkDone me
  if done or voidp(table) then
    return 
  end if
  done = (the milliSeconds > ms) and voidp(table.sitterGroup)
  return done
end
