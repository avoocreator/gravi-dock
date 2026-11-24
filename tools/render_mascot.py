"""Render maskot Gravi dengan geometri yang sama dengan app/ui/mascot.slint (satuan 200 x 164).
Menghasilkan: assets/gravi.png, assets/gravi.ico, assets/expressions.png. Jalankan: python3 tools/render_mascot.py  (butuh Pillow)."""
from PIL import Image, ImageDraw, ImageFont, ImageFilter
import os, math
OUT = os.path.join(os.path.dirname(__file__), '..', 'assets')
S = 4  # supersampling

# (eye_h, r_h, eye_w, r_w, arc, ring, lid_out, lid_in, look_x, look_y, peek, glyph, glyph_rgb) — salinan core/src/mood.rs
BLUE=(0x4d,0x7c,0xff); ORANGE=(0xff,0x9a,0x3d); YELLOW=(0xff,0xc9,0x4d); PURPLE=(0xa7,0x8b,0xfa); GRAY=(0x8b,0x93,0xa3)
def F(**k):
    d=dict(eye_h=1,r_h=1,eye_w=1,r_w=1,arc=0,ring=0,lid_out=0,lid_in=0,look_x=0,look_y=0,peek=0,glyph='',rgb=GRAY); d.update(k); return d
FACES = {
 'Idle':F(), 'Happy':F(eye_h=0,r_h=0,arc=1,glyph='✦',rgb=ORANGE), 'Surprised':F(eye_h=0,r_h=0,ring=1,glyph='!!',rgb=BLUE),
 'Sleepy':F(eye_h=.1,r_h=.1,eye_w=1.25,r_w=1.25,look_y=.2,glyph='zZ',rgb=PURPLE), 'Confused':F(r_h=.16,r_w=1.35,look_x=.1,glyph='?',rgb=BLUE),
 'Sad':F(eye_h=.8,r_h=.8,lid_out=.55,look_y=.25), 'Thinking':F(eye_h=.8,r_h=.8,lid_in=.5,look_x=.15,look_y=-.2,glyph='…'),
 'Searching':F(eye_h=.85,r_h=.85,eye_w=.9,r_w=.9,look_x=-.8,glyph=')))'), 'Working':F(eye_h=.95,r_h=.95,look_y=.1,glyph='//',rgb=BLUE),
 'Success':F(eye_h=0,r_h=0,arc=1,glyph='✦',rgb=YELLOW), 'Error':F(eye_h=.9,r_h=.45,r_w=1.15,look_x=.1,glyph='@'), 'Peek':F(peek=1),
}

def grad(size, stops, angle_vertical=True):
    w,h=size; im=Image.new('RGBA',size)
    px=im.load()
    for y in range(h):
        for x in range(w):
            t=(y/(h-1)) if angle_vertical else (x/(w-1))
            for i in range(len(stops)-1):
                (t0,c0),(t1,c1)=stops[i],stops[i+1]
                if t0<=t<=t1:
                    k=(t-t0)/(t1-t0) if t1>t0 else 0
                    px[x,y]=tuple(int(c0[j]+(c1[j]-c0[j])*k) for j in range(3))+(255,); break
    return im

def poly_gradient(base, pts, stops, bbox):
    x0,y0,x1,y1=[int(v) for v in bbox]
    g=grad((max(1,x1-x0),max(1,y1-y0)),stops)
    m=Image.new('L',base.size,0); ImageDraw.Draw(m).polygon(pts,fill=255)
    layer=Image.new('RGBA',base.size,(0,0,0,0)); layer.paste(g,(x0,y0))
    base.paste(Image.composite(layer,Image.new('RGBA',base.size,(0,0,0,0)),m),(0,0),m)

def bez(p0,p1,p2,p3,n=24):
    return [((1-t)**3*p0[0]+3*(1-t)**2*t*p1[0]+3*(1-t)*t*t*p2[0]+t**3*p3[0], (1-t)**3*p0[1]+3*(1-t)**2*t*p1[1]+3*(1-t)*t*t*p2[1]+t**3*p3[1]) for t in [i/n for i in range(n+1)]]

def render(face, px_w=400, bg=None, wiggle=0.0):
    f=FACES[face] if isinstance(face,str) else face
    u=px_w*S/200; W=int(200*u); H=int(164*u)
    im=Image.new('RGBA',(W,H),(0,0,0,0)) if bg is None else Image.new('RGBA',(W,H),bg)
    P=lambda x,y:(x*u,y*u)
    d=ImageDraw.Draw(im)
    # telinga gelap
    d.polygon([P(12+8,6+68),P(12+12,6+10),*[P(12+x,6+y) for x,y in bez((12,10),(14,3),(18,3),(22,5),8)[:0]],P(12+22,6+5),P(12+62,6+44)],fill=(0x2b,0x2a,0x38,255))
    d.polygon([P(122+58,6+68),P(122+54,6+10),P(122+44,6+5),P(122+4,6+44)],fill=(0x2b,0x2a,0x38,255))
    # sisi berwarna
    lp=[P(8+22,14)]+[P(8+x,14+y) for x,y in bez((22,0),(8,14),(0,34),(2,46))]+[P(8+x,14+y) for x,y in bez((2,46),(4,60),(16,64),(26,56))]+[P(8+30,14+20)]
    poly_gradient(im,lp,[(0,(0x3a,0x5b,0xff)),(.55,(0x3a,0xa6,0xc9)),(1,(0x5f,0xcf,0x9b))],(8*u,14*u,42*u,78*u))
    rp=[P(158+12,14)]+[P(158+x,14+y) for x,y in bez((12,0),(26,14),(34,34),(32,46))]+[P(158+x,14+y) for x,y in bez((32,46),(30,60),(18,64),(8,56))]+[P(158+4,14+20)]
    poly_gradient(im,rp,[(0,(0xff,0x5a,0x36)),(.5,(0xff,0x9a,0x3d)),(1,(0xff,0xcf,0x4d))],(158*u,14*u,192*u,78*u))
    # kepala + layar
    body=Image.new('RGBA',(W,H),(0,0,0,0))
    hd=grad((int(164*u),int(100*u)),[(0,(0xfd,0xfb,0xff)),(.6,(0xef,0xe9,0xf6)),(1,(0xd9,0xcf,0xe6))])
    hm=Image.new('L',hd.size,0); ImageDraw.Draw(hm).rounded_rectangle([0,0,hd.size[0]-1,hd.size[1]-1],radius=46*u,fill=255)
    body.paste(hd,(int(18*u),int(60*u)),hm)
    bd=ImageDraw.Draw(body)
    bd.rounded_rectangle([38*u,74*u,162*u,146*u],radius=34*u,fill=(0x20,0x1f,0x2e,255),outline=(0xcf,0xc6,0xe4,255),width=max(1,int(2*u/2)))
    # pantulan kaca
    bd.polygon([P(38+14+2,74+8+16),*[P(38+14+x,74+8+y) for x,y in bez((2,16),(6,8),(16,2),(30,2))],*[P(38+14+x,74+8+y) for x,y in bez((30,2),(22,6),(12,12),(6,18))]],fill=(255,255,255,46))
    cx=100+f['look_x']*7; cy=110+f['look_y']*5; gap=29
    for side,h,w_,in((-1,f['eye_h'],f['eye_w']),(1,f['r_h'],f['r_w'])):
        ex=cx+side*gap
        if h>0.06:
            ew,eh=12*w_,max(0.1,26*h)
            bd.rounded_rectangle([(ex-ew/2)*u,(cy-eh/2)*u,(ex+ew/2)*u,(cy+eh/2)*u],radius=min(ew,eh)/2*u,fill=(0xf4,0xf7,0xff,255),outline=(0xb7,0xc4,0xff,255),width=max(1,int(u/2)))
        lo,li=f['lid_out'],f['lid_in']
        if lo+li>0.01:
            x0=ex-10; y0=cy-15
            if side<0: pts=[(0,0),(20*li,0),(20*(1-lo),20*lo),(0,20*lo)]
            else: pts=[(20,0),(20-20*li,0),(20*lo,20*lo),(20,20*lo)]
            bd.polygon([P(x0+a,y0+b) for a,b in pts],fill=(0x20,0x1f,0x2e,255))
        if f['arc']>0.5:
            bd.line([P(ex-9,cy+5),P(ex,cy-6),P(ex+9,cy+5)],fill=(0xf4,0xf7,0xff,255),width=int(3.2*u),joint='curve')
            for q in(P(ex-9,cy+5),P(ex+9,cy+5)): bd.ellipse([q[0]-1.6*u,q[1]-1.6*u,q[0]+1.6*u,q[1]+1.6*u],fill=(0xf4,0xf7,0xff,255))
        if f['ring']>0.5: bd.ellipse([(ex-11)*u,(cy-11)*u,(ex+11)*u,(cy+11)*u],outline=(0xf4,0xf7,0xff,255),width=int(3*u))
    shift=int(f['peek']*164*0.46*u)
    im.alpha_composite(body.crop((0,0,W,H)) if shift==0 else body)
    if shift:
        clip=Image.new('RGBA',(W,H),(0,0,0,0)); clip.alpha_composite(im.crop((0,0,W,H)),(0,0)); 
        moved=Image.new('RGBA',(W,H),(0,0,0,0)); moved.paste(im,(0,shift)); im=moved
    # aksesori
    if f['glyph']:
        try: fnt=ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf',int(22*u))
        except Exception: fnt=ImageFont.load_default()
        gd=ImageDraw.Draw(im); tw=gd.textlength(f['glyph'],font=fnt)
        gd.text((W-tw-2*u,0),f['glyph'],font=fnt,fill=f['rgb']+(255,))
    return im.resize((px_w,int(px_w*164/200)),Image.LANCZOS)

if __name__=='__main__':
    os.makedirs(OUT,exist_ok=True)
    big=render('Idle',512); big.save(os.path.join(OUT,'gravi.png'))
    sq=Image.new('RGBA',(256,256),(0,0,0,0)); ic=render('Idle',236); sq.alpha_composite(ic,((256-ic.width)//2,(256-ic.height)//2))
    sq.save(os.path.join(OUT,'gravi.ico'),sizes=[(16,16),(24,24),(32,32),(48,48),(64,64),(128,128),(256,256)])
    cols,rows=4,3; cw,ch=300,260
    sheet=Image.new('RGBA',(cols*cw,rows*ch),(250,250,252,255)); sd=ImageDraw.Draw(sheet)
    try: lf=ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf',20)
    except Exception: lf=ImageFont.load_default()
    for i,n in enumerate(FACES):
        x,y=(i%cols)*cw,(i//cols)*ch
        sheet.alpha_composite(render(n,240),(x+30,y+14)); tw=sd.textlength(n,font=lf)
        sd.rounded_rectangle([x+cw/2-tw/2-14,y+212,x+cw/2+tw/2+14,y+244],radius=16,fill=(236,234,244,255)); sd.text((x+cw/2-tw/2,y+216),n,font=lf,fill=(90,90,120,255))
    sheet.convert('RGB').save(os.path.join(OUT,'expressions.png'))
    print('ok')
