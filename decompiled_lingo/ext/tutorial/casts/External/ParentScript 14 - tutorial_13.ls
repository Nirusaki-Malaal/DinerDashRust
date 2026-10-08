property done, flo
global glob

on startStep me
  done = 0
  flo = glob.managers.game.room.waiter
  if not flo.hasPayloadType(#dishes) then
    return 
  end if
  return 1
end

on checkDone me
  if done then
    return 
  end if
  done = not flo.hasPayloadType(#dishes)
  return done
end
