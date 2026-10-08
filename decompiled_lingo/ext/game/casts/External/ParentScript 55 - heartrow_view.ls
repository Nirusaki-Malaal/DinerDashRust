property origmem, pMember, alph, heartimg, heartwidth, heartheight, spacing, node, sitterGroup, width

on new me, n
  node = n
  if not voidp(node) then
    m = node.getMember()
  end if
  origmem = m
  m = member("heartrow", "gfx.people")
  heartimg = member("heart", "gfx.people").image
  heartwidth = heartimg.width
  heartheight = heartimg.height
  spacing = heartwidth - 2
  pMember = new(#bitmap, castLib("dynamic"))
  width = m.rect.width
  image = image(width, m.rect.height, 32)
  alph = image(image.rect.width, image.rect.height, 8)
  alph.fill(alph.rect, rgb(255, 255, 255))
  image.setAlpha(alph)
  pMember.image = image
  pMember.useAlpha = 1
  if not voidp(node) then
    node.setMember(pMember)
  end if
  return me
end

on setHearts me, num
  num = max(0, min(num, 10))
  left = 0
  pMember.image.setAlpha(alph)
  repeat with i = num down to 1
    r = heartimg.rect
    left = width - (i * spacing) - 2
    pMember.image.copyPixels(heartimg, rect(left, 0, left + heartwidth, heartheight), r, [#ink: 36])
  end repeat
  if not voidp(node) then
    node.setMember(pMember)
  end if
end

on setObject me, obj
  if obj = sitterGroup then
    return 
  end if
  if not voidp(sitterGroup) then
    sitterGroup.removeView(me)
  end if
  sitterGroup = obj
  if voidp(obj) then
    node.setVisible(0)
    return 
  end if
  sitterGroup.addView(me)
  node.setVisible(1)
  me.update()
end

on update me
  if voidp(sitterGroup) then
    return 
  end if
  me.setHearts(sitterGroup.mood)
end

on removeObject me, obj
  if sitterGroup = obj then
    me.setObject(VOID)
  end if
end

on destroy me
  if not voidp(pMember) then
    erase(pMember)
  end if
  if not voidp(sitterGroup) then
    sitterGroup.removeView(me)
  end if
  pMember = VOID
  if not voidp(node) then
    node = VOID
  end if
end
