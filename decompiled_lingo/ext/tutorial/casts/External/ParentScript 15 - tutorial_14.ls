property done, waitingline, ms
global glob

on startStep me
  done = 0
  waitingline = glob.managers.game.waitingGroups
  ms = the milliSeconds + 2000
  return 1
end

on checkDone me
  if done or voidp(waitingline) then
    return 
  end if
  done = (the milliSeconds > ms) and (waitingline.count = 2)
  return done
end
