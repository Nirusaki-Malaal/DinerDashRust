property done, flo
global glob

on startStep me
  done = 0
  flo = glob.managers.game.room.waiter
  if flo.payload.count then
    return 
  end if
  return 1
end

on checkDone me
  if done or voidp(flo) then
    return 
  end if
  done = flo.getPayload(#food).count = 2
  return done
end
