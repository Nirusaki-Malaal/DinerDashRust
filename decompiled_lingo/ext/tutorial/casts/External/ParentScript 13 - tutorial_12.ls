property done, table
global glob

on startStep me
  done = 0
  t = glob.managers.game.room.tables
  if t.count <> 2 then
    return 
  end if
  table = t[2]
  if not table.dirty then
    return 
  end if
  return 1
end

on checkDone me
  if done or voidp(table) then
    return 
  end if
  done = not table.dirty
  return done
end
