property groups

on new me
  groups = [:]
  return me
end

on init me
end

on getGroup me, gname
  g = groups[gname]
  if voidp(g) then
    g = script("drag_group", "scripts.events").new()
    groups[gname] = g
  end if
  return g
end

on destroy me
  call(#destroy, groups)
  groups.deleteAll()
end
