property done, flo
global glob

on startStep me
  done = 0
  flo = glob.managers.game.room.waiter
  return 1
end

on checkDone me
  if done or voidp(flo) then
    return 
  end if
  done = not flo.hasPayloadType(#ticket)
  return done
end
