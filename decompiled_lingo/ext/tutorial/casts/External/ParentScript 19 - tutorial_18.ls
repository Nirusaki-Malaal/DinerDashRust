property done, ms
global glob

on startStep me
  done = 0
  ms = the milliSeconds + 2000
  return 1
end

on checkDone me
  if done then
    return 
  end if
  done = the milliSeconds > ms
  return done
end
