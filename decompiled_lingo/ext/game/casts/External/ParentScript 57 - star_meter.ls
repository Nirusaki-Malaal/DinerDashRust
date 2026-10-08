property node, stars, capacity

on new me, n
  node = n
  stars = []
  repeat with i = 1 to 6
    stars.add(node["star_" & i])
  end repeat
  capacity = stars.count
  return me
end

on setCapacity me, num
  ct = stars.count
  capacity = max(1, min(num, ct))
  repeat with i = 1 to ct
    stars[i].setVisible(i <= num)
  end repeat
end

on setFull me, num
  repeat with i = 1 to capacity
    if i <= num then
      stars[i].setAnimation("full")
      next repeat
    end if
    stars[i].setAnimation("empty")
  end repeat
end

on destroy me
  stars.deleteAll()
  node = VOID
end
