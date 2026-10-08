property done, flo
global glob

on startStep me
  done = 0
  flo = glob.managers.game.room.waiter
  return flo.payload.count = 0
end

on checkDone me
  if done then
    return 
  end if
  done = flo.hasPayloadType(#food)
  return done
end
