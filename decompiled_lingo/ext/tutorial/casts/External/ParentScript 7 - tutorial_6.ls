property done, kitchenTimers
global glob

on startStep me
  done = 0
  kitchenTimers = glob.managers.game.room.kitchen.timers
  return kitchenTimers.count > 0
end

on checkDone me
  if done or voidp(kitchenTimers) then
    return 
  end if
  done = kitchenTimers.count = 0
  return done
end
