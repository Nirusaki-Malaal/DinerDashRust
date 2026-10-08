on new me, sm, dat
  if not voidp(dat) then
    if not dat[#noSprite] or voidp(dat[#member]) or (dat[#type] = #Branch) then
      return script("node2d_branch").new(sm, dat)
    end if
  end if
  return script("node2d_leaf").new(sm, dat)
end
