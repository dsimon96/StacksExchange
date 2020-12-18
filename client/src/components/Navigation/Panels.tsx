import React, { PropsWithChildren } from 'react';

interface IProps {
    index: number;
};

export const Panels: React.FC<PropsWithChildren<IProps>> = (props) => {
    return <>{React.Children.toArray(props.children)[props.index]}</>
}