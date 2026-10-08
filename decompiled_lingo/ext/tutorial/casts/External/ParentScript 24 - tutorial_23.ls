property done, tables, flo
global glob

on startStep me
  done = 0
  flo = glob.managers.game.room.waiter
  tables = glob.managers.game.room.tables
  if tables.count <> 2 then
    return 
  end if
  repeat with t in tables
    if not t.dirty then
      return 
    end if
  end repeat
  return 1
end

on checkDone me
  if done then
    return 
  end if
  clean = 1
  repeat with t in tables
    clean = clean and not t.dirty
  end repeat
  done = (flo.payload.count = 0) and clean
  return done
end
