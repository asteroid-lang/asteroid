''' Conways Game of Life - no wrap around'''

import os
import time
import random
import copy


##########################################################################
import platform

def clear_cmd():

    if platform.win32_ver()[0]:
        return 'cls'
    else:
        return 'clear'


##########################################################################
def display_array(ar):
    "clear the screen, display the contents of an array, wait for .5sec"
    os.system(clear_cmd())

    rows = len(ar)    # grab the rows   
    
    if rows == 0:
        raise ValueError("Array contains no data")     
        
    cols = len(ar[0]) # grab the columns - indices start at 0!

    for i in range(rows):
        for j in range(cols):
            print(ar[i][j],end=' ') # no carriage return, space separated
        print()

    time.sleep(.5)

##########################################################################
def init_map(xdim, ydim):
    thresh = 0.8
    return [[1 if random.random() > thresh else 0 for y in range(ydim)] 
            for x in range(xdim)]

##########################################################################
def array_shape(arr):
    xdim = len(arr)
    ydim = len(arr[0])
    return xdim, ydim

##########################################################################
def compute_new_state(map1, map2):
    'Compute a new map2 from an old map1'

    xdim, ydim = array_shape(map1)

    # compute our new map
    for x in range(xdim):
        for y in range(ydim):
            # probe the map and count the neighbors
            ct = 0
            ct += map1[x-1][y-1] if x-1 >= 0   and y-1 >= 0   else 0
            ct += map1[x][y-1]   if                 y-1 >= 0  else 0
            ct += map1[x+1][y-1] if x+1 < xdim and y-1 >= 0   else 0
            ct += map1[x+1][y]   if x+1 < xdim                else 0
            ct += map1[x+1][y+1] if x+1 < xdim and y+1 < ydim else 0
            ct += map1[x][y+1]   if                y+1 < ydim else 0
            ct += map1[x-1][y+1] if x-1 >= 0   and y+1 < ydim else 0
            ct += map1[x-1][y]   if x-1 >= 0                  else 0
            # update the output map
            # element is alive
            if map1[x][y]==1 and ct>1 and ct<4:
                map2[x][y]=1
            else:
                map2[x][y]=0

            # element is dead
            if map1[x][y]==0 and ct==3:
                map2[x][y]=1

##########################################################################
def show_map(map):
    ''' Display our map to the terminal using asterisk for live cells 
        and space for dead ones.'''

    xdim, ydim = array_shape(map)

    # need to do a deep copy because we are modifying the map
    display_map = copy.deepcopy(map) 

    for x in range(xdim):
        for y in range(ydim):
            if display_map[x][y] > 0:
                display_map[x][y] = '*'
            else:
                display_map[x][y] = ' '

    display_array(display_map)

##########################################################################
def life(maxgen=10, xdim=60, ydim=40):

    # NOTE: y horizontal, x vertical - this was ported from Perl and there
    #       got stuck this way...
    map1 = init_map(xdim, ydim)
    map2 = init_map(xdim, ydim)

    # show the initial map
    show_map(map1)

    for gen in range(maxgen):
        compute_new_state(map1, map2)
        show_map(map2)
        # implement double buffering
        map1, map2 = map2, map1 # swap the maps

##########################################################################
if __name__ == "__main__":
    life(200, 30, 40)

