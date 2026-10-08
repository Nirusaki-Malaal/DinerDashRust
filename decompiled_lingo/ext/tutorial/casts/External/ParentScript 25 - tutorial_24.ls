property done, ms
global glob

on startStep me
  ms = the milliSeconds + 2000
  done = 0
  return 1
end

on checkDone me
  if done then
    return 
  end if
  done = the milliSeconds > ms
  return done
end
