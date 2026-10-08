property table, done
global glob

on startStep me
  done = 0
  t = glob.managers.game.room.tables
  if t.count <> 2 then
    table = VOID
    return 
  end if
  table = t[2]
  return 1
end

on checkDone me
  if done or voidp(table) then
    return 
  end if
  return not voidp(table.sitterGroup)
end
